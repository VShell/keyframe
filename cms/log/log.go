package log

import (
	"context"
	"net/http"

	"github.com/google/uuid"
	"go.uber.org/zap"
	"keyframe.alterednarrative.net/cms/httpcontext"
)

type requestIDTokenType struct{}

var requestIDToken = requestIDTokenType{}

var logger *zap.Logger
var httpLogger *zap.Logger

func init() {
	var err error
	logger, err = zap.NewDevelopment()
	if err != nil {
		panic(err)
	}
	zap.ReplaceGlobals(logger)
	httpLogger = logger.With(zap.String("protocol", "http"))
}

func genrid() string {
	return uuid.Must(uuid.NewRandom()).String()
}

type httpHandler struct {
	next http.Handler
}

func (h httpHandler) ServeHTTP(rw http.ResponseWriter, req *http.Request) {
	var requestID string
	if httpcontext.IsTrustedClient(req.Context()) {
		requestID = req.Header.Get("X-Request-ID")
	}
	if requestID == "" {
		requestID = genrid()
	}

	req = req.Clone(context.WithValue(req.Context(), requestIDToken, requestID))
	h.next.ServeHTTP(rw, req)
}

func HttpMiddleware(next http.Handler) http.Handler {
	return httpHandler{
		next: next,
	}
}

func Logger() *zap.Logger {
	return logger
}

func HttpLogger(ctx context.Context) *zap.Logger {
	logger := httpLogger
	if requestID, ok := ctx.Value(requestIDToken).(string); ok {
		logger = logger.With(zap.String("request-id", requestID))
	}
	return logger
}
