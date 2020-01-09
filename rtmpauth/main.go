package main

import (
	"bufio"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
)

type Config struct {
	sync.RWMutex
	UsersFromKeys map[string]string
}

func main() {
	config := &Config{}

	ReadConfig(config)

	reload := make(chan os.Signal, 1)
	signal.Notify(reload, syscall.SIGUSR1)
	go func() {
		for {
			<-reload
			ReadConfig(config)
		}
	}()

	http.HandleFunc("/on_publish", func(w http.ResponseWriter, r *http.Request) {
		config.RLock()
		user, ok := config.UsersFromKeys[r.FormValue("name")]
		config.RUnlock()
		if ok {
			w.Header().Set("Location", user)
			w.WriteHeader(http.StatusTemporaryRedirect)
		} else {
			w.WriteHeader(http.StatusNotFound)
		}
	})

	log.Fatal(http.ListenAndServe("127.0.0.1:1337", nil))
}

func ReadConfig(config *Config) {
	config.Lock()
	defer config.Unlock()

	config.UsersFromKeys = make(map[string]string)

	file, err := os.Open("/var/lib/rtmpauth/users")
	if err != nil {
		log.Fatal(err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		parts := strings.SplitN(scanner.Text(), ":", 2)
		if len(parts) != 2 {
			log.Printf("config line invalid: %v", parts[0])
			continue
		}
		config.UsersFromKeys[parts[1]] = parts[0]
	}
	if err := scanner.Err(); err != nil {
		log.Fatal(err)
	}
}
