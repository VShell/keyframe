package configuration

import (
	"bytes"
	"net"

	"go.uber.org/zap"
)

type TrustedProxies struct {
	TrustedProxies []string `toml:"trusted-proxies"`
}

func (tp TrustedProxies) ParseTrustedProxies(logger *zap.Logger) []*net.IPNet {
	trustedProxies := make([]*net.IPNet, len(tp.TrustedProxies))
	for i, proxy := range tp.TrustedProxies {
		_, ipnet, err := net.ParseCIDR(proxy)
		if err != nil {
			ip := net.ParseIP(proxy)
			if ip == nil {
				logger.Fatal(
					"Couldn't parse proxy as CIDR or IP",
					zap.String("proxy", proxy),
				)
			}
			ipnet = &net.IPNet{
				IP:   ip,
				Mask: net.IPMask(bytes.Repeat([]byte{0xff}, len(ip))),
			}
		}
		trustedProxies[i] = ipnet
	}
	return trustedProxies
}
