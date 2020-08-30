package httpcontext

import (
	"context"
)

type trustedClientKeyType struct{}

var trustedClientKey = trustedClientKeyType{}

func WithTrustedClient(ctx context.Context) context.Context {
	return context.WithValue(ctx, trustedClientKey, struct{}{})
}

func IsTrustedClient(ctx context.Context) bool {
	_, ok := ctx.Value(trustedClientKey).(struct{})
	return ok
}
