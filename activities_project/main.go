package main

import (
	"io"
	"os"
	"path/filepath"
	"strings"
	"time"

	"example.com/monitoring-agent/configmanager"

	logrus "github.com/sirupsen/logrus"
)

type PlainMessageFormatter struct{}

func (f *PlainMessageFormatter) Format(entry *logrus.Entry) ([]byte, error) {
	return []byte(entry.Message + "\n"), nil
}

func parseArgs() (metric string, once bool, ok bool) {
	if len(os.Args) < 2 {
		return "", false, false
	}

	once = false
	metricParts := make([]string, 0, len(os.Args)-1)

	for i := 1; i < len(os.Args); i++ {
		switch os.Args[i] {
		case "--once":
			once = true
		default:
			metricParts = append(metricParts, os.Args[i])
		}
	}

	metric = strings.TrimSpace(strings.Join(metricParts, " "))
	if metric == "" {
		return "", once, false
	}

	return metric, once, true
}

func main() {
	metric, once, ok := parseArgs()
	logger := logrus.New()
	if !ok {
		logger.SetOutput(os.Stdout)
		logger.Errorf("Provide next arguments: <metric> [--once]")
		os.Exit(1)
	}

	config, err := configmanager.LoadConfig()
	logFilePath := config.LogFilePath
	if err != nil {
		logger.SetOutput(os.Stdout)
		logger.Warnf("loading config failed, using default log path: %v", err)
	}

	if err := os.MkdirAll(filepath.Dir(logFilePath), 0o755); err != nil {
		logger.SetOutput(os.Stdout)
		logger.Warnf("creating log directory failed: %v", err)
	}

	logFile, err := os.OpenFile(logFilePath, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)
	if err != nil {
		logger.SetOutput(os.Stdout)
		logger.Errorf("opening log file failed: %v", err)
	} else {
		defer logFile.Close()
		logger.SetOutput(io.MultiWriter(os.Stdout, logFile))
	}

	logger.SetFormatter(&PlainMessageFormatter{})
	logMetric := func() {
		logger.Infof("%s", metric)
	}

	if once {
		logMetric()
		return
	}

	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	logMetric()
	for range ticker.C {
		logMetric()
	}
}
