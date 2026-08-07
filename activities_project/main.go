package main

import (
	"io"
	"os"
    logrus "github.com/sirupsen/logrus"
)


type PlainMessageFormatter struct{}


func (f *PlainMessageFormatter) Format(entry *logrus.Entry) ([]byte, error) {
    return []byte(entry.Message + "\n"), nil
}


func main() {
	_ = os.MkdirAll("logs", 0o755)

    logFile, err := os.OpenFile("logs/agent.log", os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o644)

    if err != nil {
        panic(err)
    }
    defer logFile.Close()
	logger := logrus.New()
	logger.SetOutput(io.MultiWriter(os.Stdout, logFile))


	logger.SetFormatter(&PlainMessageFormatter{})

	if len(os.Args) < 2 {
		logger.Errorf("Provide next arguments: <arg1>")
		os.Exit(1)
	}

	arg1 := os.Args[1]

	logger.Infof("%s", arg1)
}