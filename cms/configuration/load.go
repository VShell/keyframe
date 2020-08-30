package configuration

import (
	"os"

	"github.com/pelletier/go-toml"
	"go.uber.org/zap"
)

func Load(logger *zap.Logger, config interface{}) {
	if len(os.Args) < 2 {
		logger.Fatal("Configuration file not passed on command line")
	}

	configFile, err := os.Open(os.Args[1])
	if err != nil {
		logger.Fatal(
			"Couldn't open configuration file",
			zap.String("filename", os.Args[1]),
			zap.Error(err),
		)
	}
	decoder := toml.NewDecoder(configFile)
	err = decoder.Decode(config)
	if err != nil {
		logger.Fatal(
			"Couldn't parse configuration file",
			zap.String("filename", os.Args[1]),
			zap.Error(err),
		)
	}
}
