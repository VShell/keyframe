package httpserver

import (
	"context"
	"net"
	"net/http"
	"os"
	"os/signal"

	"go.uber.org/zap"
	"keyframe.alterednarrative.net/cms/httpcontext"
	"keyframe.alterednarrative.net/cms/log"
)

func Serve(logger *zap.Logger, trustedProxies []*net.IPNet, handler http.Handler) {
	listener, err := net.FileListener(os.Stdin)
	if err != nil {
		logger.Fatal("Couldn't open stdin as a listener", zap.Error(err))
	}

	server := http.Server{}
	server.BaseContext = func(l net.Listener) context.Context {
		ctx := context.Background()
		if _, ok := l.(*net.UnixListener); ok {
			ctx = httpcontext.WithTrustedClient(ctx)
		} else if _, ok := l.(*net.TCPListener); ok {
			server.ConnContext = tcpConnContext(logger, trustedProxies)
		}
		return ctx
	}

	handler = log.HttpMiddleware(handler)

	idleConnsClosed := make(chan struct{})
	go func() {
		sig := make(chan os.Signal, 1)
		signal.Notify(sig, os.Interrupt, os.Kill)
		<-sig
		if err := server.Shutdown(context.Background()); err != nil {
			logger.Error("Couldn't cleanly shut down HTTP server", zap.Error(err))
		}
		close(idleConnsClosed)
	}()

	if err := http.Serve(listener, handler); err != http.ErrServerClosed {
		logger.Fatal("HTTP server returned an error", zap.Error(err))
	}
	<-idleConnsClosed
}

func tcpConnContext(logger *zap.Logger, trustedProxies []*net.IPNet) func(context.Context, net.Conn) context.Context {
	return func(ctx context.Context, c net.Conn) context.Context {
		host, _, err := net.SplitHostPort(c.RemoteAddr().String())
		if err != nil {
			logger.DPanic("Error processing remote address", zap.Error(err), zap.String("address", c.RemoteAddr().String()))
			return ctx
		}
		ip := net.ParseIP(host)
		for _, net := range trustedProxies {
			if net.Contains(ip) {
				ctx = httpcontext.WithTrustedClient(ctx)
				return ctx
			}
		}
		return ctx
	}
}
