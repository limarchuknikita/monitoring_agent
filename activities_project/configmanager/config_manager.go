package configmanager

import (
	"bufio"
	"errors"
	"os"
	"path/filepath"
	"strings"
)

const DefaultLogFilePath = "./logs/agent.log"

type Config struct {
	LogFilePath string
}

func LoadConfig() (Config, error) {
	exePath, err := os.Executable()
	if err != nil {
		return Config{LogFilePath: DefaultLogFilePath}, err
	}

	exeDir := filepath.Dir(exePath)
	settingsPath := findSettingsPath(exeDir)
	if settingsPath == "" {
		return Config{LogFilePath: filepath.Clean(filepath.Join(exeDir, DefaultLogFilePath))}, errors.New("settings.toml not found")
	}

	logFilePath, err := readLogFilePath(settingsPath)
	if err != nil {
		return Config{LogFilePath: filepath.Clean(filepath.Join(exeDir, DefaultLogFilePath))}, err
	}

	if logFilePath == "" {
		logFilePath = DefaultLogFilePath
	}

	if !filepath.IsAbs(logFilePath) {
		logFilePath = filepath.Join(exeDir, logFilePath)
	}

	return Config{LogFilePath: filepath.Clean(logFilePath)}, nil
}

func findSettingsPath(exeDir string) string {
	candidates := []string{
		filepath.Join(exeDir, "settings.toml"),
		filepath.Join(filepath.Dir(exeDir), "settings.toml"),
		"settings.toml",
	}

	for _, candidate := range candidates {
		if _, err := os.Stat(candidate); err == nil {
			return candidate
		}
	}

	return ""
}

func readLogFilePath(settingsPath string) (string, error) {
	file, err := os.Open(settingsPath)
	if err != nil {
		return "", err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			continue
		}

		key := strings.TrimSpace(parts[0])
		value := strings.TrimSpace(parts[1])
		value = strings.Trim(value, "\"'")

		if key == "log_file_path" {
			return value, nil
		}
	}

	if err := scanner.Err(); err != nil {
		return "", err
	}

	return "", nil
}
