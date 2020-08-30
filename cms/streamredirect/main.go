package main

import (
	"database/sql"
	"encoding/json"
	"net/http"
	"net/url"

	_ "github.com/mattn/go-sqlite3"
	"go.uber.org/zap"
	"keyframe.alterednarrative.net/cms/configuration"
	"keyframe.alterednarrative.net/cms/httpserver"
	"keyframe.alterednarrative.net/cms/log"
)

type Config struct {
	configuration.TrustedProxies
	Database string `toml:"database"`
}

type Redirector struct {
	Logger    *zap.Logger
	Database  *sql.DB
	Redirects map[string]string
}

func main() {
	logger := log.Logger()
	var config Config
	configuration.Load(logger, &config)
	trustedProxies := config.ParseTrustedProxies(logger)

	db, err := sql.Open("sqlite3", config.Database)
	if err != nil {
		logger.Fatal(
			"Error opening database",
			zap.Error(err),
		)
	}
	defer db.Close()

	redirector := Redirector{
		Logger:    logger,
		Database:  db,
		Redirects: map[string]string{},
	}

	err = redirector.LoadDatabase()
	if err != nil {
		logger.Fatal(
			"Error reading state from database",
			zap.Error(err),
		)
	}

	mux := http.NewServeMux()
	mux.Handle("/", redirector.RedirectHandler())
	mux.Handle("/api/v1/ingestd-notify", redirector.IngestdNotifyHandler())

	httpserver.Serve(logger, trustedProxies, mux)
}

func (r *Redirector) LoadDatabase() error {
	rows, err := r.Database.Query("SELECT streams.mpd_url, stream_redirects.mpd_url FROM streams INNER JOIN stream_redirects ON stream_id = id")
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var mpd_url string
		var ingestd_mpd_url string
		err := rows.Scan(&mpd_url, &ingestd_mpd_url)
		if err != nil {
			return err
		}
		r.Redirects[mpd_url] = ingestd_mpd_url
	}
	err = rows.Err()
	if err != nil {
		return err
	}

	return nil
}

func (r *Redirector) RedirectHandler() http.Handler {
	return http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		r.Logger.Debug("redirect handler")
		var url url.URL
		url.Scheme = "https"
		url.Host = req.Host
		url.Path = req.URL.Path

		r.Logger.Debug("redirect from", zap.String("url", url.String()))

		ingestd_mpd_url, ok := r.Redirects[url.String()]
		if !ok {
			http.Error(rw, http.StatusText(http.StatusNotFound), http.StatusNotFound)
			return
		}
		http.Redirect(rw, req, ingestd_mpd_url, http.StatusTemporaryRedirect)
	})
}

type NotifyBody struct {
	Token  string `json:"token"`
	Online bool   `json:"online"`
	MpdUrl string `json:"mpd_url"`
}

func (r *Redirector) IngestdNotifyHandler() http.Handler {
	return http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		r.Logger.Debug("ingestd-notify")
		var body NotifyBody
		err := json.NewDecoder(req.Body).Decode(&body)
		if err != nil {
			r.Logger.Debug("couldn't decode json", zap.Error(err))
			http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}

		tx, err := r.Database.Begin()
		if err != nil {
			http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}
		defer func() {
			if err != nil {
				r.Logger.Debug("some error happened", zap.Error(err))
				tx.Rollback()
			}
		}()

		row := tx.QueryRow(
			"SELECT streams.id, streams.mpd_url, stream_redirects.ingestd_token, ingestd_tokens.id FROM streams INNER JOIN ingestd_tokens ON ingestd_tokens.stream_id = streams.id LEFT JOIN stream_redirects ON stream_redirects.stream_id = streams.id WHERE ingestd_tokens.token = ?",
			body.Token,
		)
		var streamID int64
		var mpdUrl string
		var currentIngestdTokenID sql.NullInt64
		var ingestdTokenID int64
		err = row.Scan(&streamID, &mpdUrl, &currentIngestdTokenID, &ingestdTokenID)
		if err != nil {
			http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}

		if !body.Online && currentIngestdTokenID.Valid && currentIngestdTokenID.Int64 != ingestdTokenID {
			http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}

		if body.Online && !currentIngestdTokenID.Valid {
			_, err = tx.Exec(
				"INSERT INTO stream_redirects (stream_id, mpd_url, ingestd_token) VALUES (?, ?, ?)",
				streamID,
				body.MpdUrl,
				ingestdTokenID,
			)
			if err != nil {
				http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
				return
			}
		} else if body.Online {
			_, err = tx.Exec(
				"UPDATE stream_redirects SET mpd_url=?, ingestd_token=? WHERE stream_id=?",
				body.MpdUrl,
				ingestdTokenID,
				streamID,
			)
		} else {
			_, err = tx.Exec("DELETE FROM stream_redirects WHERE stream_id = ?", streamID)
			if err != nil {
				http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
				return
			}
		}

		err = tx.Commit()
		if err != nil {
			http.Error(rw, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
			return
		}

		if body.Online {
			r.Redirects[mpdUrl] = body.MpdUrl
		} else {
			delete(r.Redirects, mpdUrl)
		}

		rw.WriteHeader(http.StatusNoContent)
	})
}
