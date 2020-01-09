package main

import (
	"bufio"
	"context"
	"encoding/xml"
	"flag"
	"fmt"
	"github.com/google/uuid"
	"gosrc.io/xmpp"
	"gosrc.io/xmpp/stanza"
	"log"
	"os"
	"strings"
	"sync"
	"time"
)

type Room struct {
	Jid      string
	AdminJid string
}

type RoomStatus struct {
	Code    string
	Message string
}

func main() {
	stanza.TypeRegistry.MapExtension(stanza.PKTPresence, xml.Name{Space: "http://jabber.org/protocol/muc#user", Local: "x"}, XMucUserPresence{})
	stanza.TypeRegistry.MapExtension(stanza.PKTIQ, xml.Name{Space: "http://jabber.org/protocol/muc#admin", Local: "query"}, AdminQuery{})

	address := flag.String("address", "", "XMPP server address")
	jid := flag.String("jid", "", "XMPP JID")
	password := flag.String("password", "", "XMPP password")
	ensureFilename := flag.String("ensure", "", "File of rooms to ensure exist, separated by newline")
	removeFilename := flag.String("remove", "", "File of rooms to remove, separated by newline")
	flag.Parse()

	ensure := readRoomsFromFile(*ensureFilename)
	remove := readLinesFromFile(*removeFilename)

	if len(ensure)+len(remove) == 0 {
		return
	}

	config := xmpp.Config{
		TransportConfiguration: xmpp.TransportConfiguration{
			Address: *address,
		},
		Jid:        *jid,
		Credential: xmpp.Password(*password),
		Insecure:   true,
	}

	router := xmpp.NewRouter()

	client, err := xmpp.NewClient(config, router)
	if err != nil {
		log.Fatalf("%+v", err)
	}

	// If you pass the client to a connection manager, it will handle the reconnect policy
	// for you automatically.
	wg, statuses, postConnect := postConnect(router, ensure, remove)
	sm := xmpp.NewStreamManager(client, postConnect)

	go func() {
		wg.Wait()
		sm.Stop()
	}()

	err = sm.Run()
	if err != nil {
		log.Fatalf("%+v", err)
	}

	exitCode := 0
	for room, status := range statuses {
		if status.Code == "error" {
			exitCode = 3
		}
		fmt.Printf("%s\t%s\t%s\n", room, status.Code, status.Message)
	}
	os.Exit(exitCode)
}

func readLinesFromFile(filename string) []string {
	if filename == "" {
		return []string{}
	}

	file, err := os.Open(filename)
	if err != nil {
		log.Fatalf("%+v", err)
	}
	defer file.Close()

	var lines []string
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line != "" {
			lines = append(lines, scanner.Text())
		}
	}
	if scanner.Err() != nil {
		log.Fatalf("%+v", err)
	}
	return lines
}

func readRoomsFromFile(filename string) []Room {
	lines := readLinesFromFile(filename)
	rooms := []Room{}
	for _, line := range lines {
		parts := strings.Split(line, "\t")
		if len(parts) != 2 {
			log.Printf("room line invalid: %v", line)
			continue
		}
		rooms = append(rooms, Room{
			Jid:      parts[0],
			AdminJid: parts[1],
		})
	}
	return rooms
}

type OwnerQuery struct {
	XMLName xml.Name `xml:"http://jabber.org/protocol/muc#owner query"`
	X       *X       `xml:",omitempty"`
	Destroy *Destroy `xml:",omitempty"`
}

type Destroy struct {
	XMLName xml.Name `xml:"destroy"`
}

func (q OwnerQuery) Namespace() string {
	return "http://jabber.org/protocol/muc#owner"
}

type X struct {
	XMLName xml.Name `xml:"jabber:x:data x"`
	Type    string   `xml:"type,attr"`
	Fields  []XField `xml:"field"`
}

type XField struct {
	XMLName xml.Name `xml:"field"`
	Var     string   `xml:"var,attr"`
	Value   string   `xml:"value"`
}

type AdminQuery struct {
	XMLName xml.Name `xml:"http://jabber.org/protocol/muc#admin query"`
	Items   []Item   `xml:"item"`
}

func (q AdminQuery) Namespace() string {
	return "http://jabber.org/protocol/muc#admin"
}

type Item struct {
	XMLName     xml.Name `xml:"item"`
	Affiliation string   `xml:"affiliation,attr"`
	Jid         string   `xml:"jid,attr"`
}

type XMucPresence struct {
	XMLName xml.Name `xml:"http://jabber.org/protocol/muc x"`
}

type XMucUserPresence struct {
	XMLName xml.Name `xml:"http://jabber.org/protocol/muc#user x"`
	Status  []Status `xml:"status"`
}

type Status struct {
	XMLName xml.Name `xml:"status"`
	Code    uint     `xml:"code,attr"`
}

type FromMatcher struct {
	From string
}

func (m FromMatcher) Match(p stanza.Packet, rm *xmpp.RouteMatch) bool {
	presence, ok := p.(stanza.Presence)
	return ok && presence.From == m.From
}

func postConnect(router *xmpp.Router, ensure []Room, remove []string) (*sync.WaitGroup, map[string]RoomStatus, func(xmpp.Sender)) {
	var wg sync.WaitGroup
	wg.Add(len(ensure) + len(remove))

	statuslock := &sync.Mutex{}
	statuses := make(map[string]RoomStatus)

	firstRun := true
	return &wg, statuses, func(s xmpp.Sender) {
		if !firstRun {
			return
		}
		firstRun = false

		for _, room := range ensure {
			var status RoomStatus

			uuid, err := uuid.NewRandom()
			if err != nil {
				log.Fatalf("%+v", err)
			}
			presence := stanza.NewPresence(stanza.Attrs{To: room.Jid + "/" + uuid.String()})
			presence.Extensions = []stanza.PresExtension{XMucPresence{}}
			s.Send(presence)

			// wait for presence saying we've either joined the room or it's created
			presenceChannel := make(chan stanza.Packet, 1)
			router.NewRoute().Packet("presence").AddMatcher(FromMatcher{From: room.Jid + "/" + uuid.String()}).HandlerFunc(func(s xmpp.Sender, p stanza.Packet) {
				presenceChannel <- p
			})

			go func() {
				select {
				case <-time.After(30 * time.Second):
					status.Code = "error"
					status.Message = "timeout"
				case p := <-presenceChannel:
					// did the room already exist?
					// that is, does status not include 201
					presence, ok := p.(stanza.Presence)
					if !ok {
						status.Code = "error"
						status.Message = "presence couldn't be read"
						break
					}
					if presence.Type == "error" {
						status.Code = "error"
						status.Message = string(presence.Error.Type) + " - " + presence.Error.Reason + " - " + presence.Error.Text
						break
					}

					var mucUserPresence XMucUserPresence
					if !presence.Get(&mucUserPresence) {
						status.Code = "error"
						status.Message = "presence isn't a MUC presence"
						break
					}

					created := false
					for _, status := range mucUserPresence.Status {
						if status.Code == 201 {
							created = true
							break
						}
					}

					if created {
						iq := stanza.NewIQ(stanza.Attrs{To: room.Jid, Type: stanza.IQTypeSet})
						iq.Payload = OwnerQuery{X: &X{Type: "submit", Fields: []XField{
							XField{Var: "FORM_TYPE", Value: "http://jabber.org/protocol/muc#roomconfig"},
							XField{Var: "muc#roomconfig_persistentroom", Value: "true"},
						}}}
						ctx, _ := context.WithTimeout(context.Background(), 30*time.Second)
						result, err := s.SendIQ(ctx, iq)
						if err != nil {
							status.Code = "error"
							status.Message = err.Error()
							break
						}

						select {
						case <-ctx.Done():
							status.Code = "error"
							status.Message = "timeout"
						case iqresult := <-result:
							if iqresult.Error != nil {
								status.Code = "error"
								status.Message = string(iqresult.Error.Type) + " - " + iqresult.Error.Reason + " - " + iqresult.Error.Text
								break
							}

							addAdmin(s, &status, room)
							status.Code = "added"
						}
					} else if !isAdmin(s, &status, room) {
						addAdmin(s, &status, room)
						status.Code = "updated"
					} else if status.Code == "" {
						status.Code = "noop"
					}
				}

				statuslock.Lock()
				statuses[room.Jid] = status
				statuslock.Unlock()
				wg.Done()
			}()
		}

		for _, room := range remove {
			var status RoomStatus

			iq := stanza.NewIQ(stanza.Attrs{To: room, Type: stanza.IQTypeSet})
			iq.Payload = OwnerQuery{Destroy: &Destroy{}}
			ctx, _ := context.WithTimeout(context.Background(), 30*time.Second)
			result, err := s.SendIQ(ctx, iq)
			if err != nil {
				status.Code = "error"
				status.Message = err.Error()
				wg.Done()
				continue
			}

			go func() {
				select {
				case <-ctx.Done():
					status.Code = "error"
					status.Message = "timeout"
				case iqresult := <-result:
					if iqresult.Error != nil {
						if iqresult.Error.Type == "cancel" && (iqresult.Error.Reason == "gone" || iqresult.Error.Reason == "item-not-found") {
							status.Code = "noop"
						} else {
							status.Code = "error"
							status.Message = string(iqresult.Error.Type) + " - " + iqresult.Error.Reason + " - " + iqresult.Error.Text
						}
						break
					}
					status.Code = "removed"
				}

				statuslock.Lock()
				statuses[room] = status
				statuslock.Unlock()
				wg.Done()
			}()
		}
	}
}

func isAdmin(s xmpp.Sender, status *RoomStatus, room Room) bool {
	iq := stanza.NewIQ(stanza.Attrs{To: room.Jid, Type: stanza.IQTypeGet})
	iq.Payload = AdminQuery{Items: []Item{Item{Affiliation: "admin"}}}
	ctx, _ := context.WithTimeout(context.Background(), 30*time.Second)
	result, err := s.SendIQ(ctx, iq)
	if err != nil {
		status.Code = "error"
		status.Message = err.Error()
		// don't try to add the user as admin
		return true
	}

	select {
	case <-ctx.Done():
		status.Code = "error"
		status.Message = "timeout"
		return true
	case iqresult := <-result:
		if iqresult.Error != nil {
			status.Code = "error"
			status.Message = string(iqresult.Error.Type) + " - " + iqresult.Error.Reason + " - " + iqresult.Error.Text
			return true
		}
		queryresult, ok := iqresult.Payload.(*AdminQuery)
		if !ok {
			status.Code = "error"
			status.Message = "response isn't a muc#admin response"
			return true
		}
		for _, item := range queryresult.Items {
			if item.Jid == room.AdminJid {
				return true
			}
		}
		return false
	}
}

func addAdmin(s xmpp.Sender, status *RoomStatus, room Room) {
	iq := stanza.NewIQ(stanza.Attrs{To: room.Jid, Type: stanza.IQTypeSet})
	iq.Payload = AdminQuery{Items: []Item{Item{Affiliation: "admin", Jid: room.AdminJid}}}
	ctx, _ := context.WithTimeout(context.Background(), 30*time.Second)
	result, err := s.SendIQ(ctx, iq)
	if err != nil {
		status.Code = "error"
		status.Message = err.Error()
		return
	}

	select {
	case <-ctx.Done():
		status.Code = "error"
		status.Message = "timeout"
	case iqresult := <-result:
		if iqresult.Error != nil {
			status.Code = "error"
			status.Message = string(iqresult.Error.Type) + " - " + iqresult.Error.Reason + " - " + iqresult.Error.Text
		}
	}
}
