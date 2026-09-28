// Command rfc is the Go installer shim for the canonical Rust executable.
package main

import (
	"archive/tar"
	"archive/zip"
	"bytes"
	"compress/gzip"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"strings"
	"time"
)

const (
	releasesURL = "https://github.com/gkk-dev-ops/rfc-cli/releases"
	maxDownload = 100 << 20
)

func main() {
	binary, err := installedBinary()
	if err != nil {
		fmt.Fprintln(os.Stderr, "rfc bootstrap:", err)
		os.Exit(1)
	}
	command := exec.Command(binary, os.Args[1:]...)
	command.Stdin, command.Stdout, command.Stderr = os.Stdin, os.Stdout, os.Stderr
	if err := command.Run(); err != nil {
		var exitError *exec.ExitError
		if errors.As(err, &exitError) {
			os.Exit(exitError.ExitCode())
		}
		fmt.Fprintln(os.Stderr, "rfc bootstrap:", err)
		os.Exit(1)
	}
}

func installedBinary() (string, error) {
	cacheRoot, err := os.UserCacheDir()
	if err != nil {
		return "", fmt.Errorf("resolve cache directory: %w", err)
	}
	name := "rfc"
	if runtime.GOOS == "windows" {
		name += ".exe"
	}
	binary := filepath.Join(cacheRoot, "rfc-agent-cli", "bootstrap", name)
	if _, err := os.Stat(binary); err == nil && os.Getenv("RFC_BOOTSTRAP_REFRESH") != "1" {
		return binary, nil
	}
	asset, err := assetName()
	if err != nil {
		return "", err
	}
	if err := downloadAndVerify(asset, binary); err != nil {
		return "", err
	}
	return binary, nil
}

func assetName() (string, error) {
	architecture := map[string]string{"amd64": "x86_64", "arm64": "aarch64"}[runtime.GOARCH]
	if architecture == "" {
		return "", fmt.Errorf("unsupported architecture %s", runtime.GOARCH)
	}
	switch runtime.GOOS {
	case "darwin":
		return fmt.Sprintf("rfc-agent-cli-%s-apple-darwin.tar.gz", architecture), nil
	case "linux":
		return fmt.Sprintf("rfc-agent-cli-%s-unknown-linux-gnu.tar.gz", architecture), nil
	case "windows":
		if architecture == "x86_64" {
			return "rfc-agent-cli-x86_64-pc-windows-msvc.zip", nil
		}
	}
	return "", fmt.Errorf("unsupported platform %s/%s", runtime.GOOS, runtime.GOARCH)
}

func downloadAndVerify(asset, destination string) error {
	ctx, cancel := context.WithTimeout(context.Background(), 90*time.Second)
	defer cancel()
	base := releaseDownloadBase()
	checksums, err := download(ctx, base+"/"+asset+".sha256")
	if err != nil {
		return fmt.Errorf("download checksums: %w", err)
	}
	expected, err := checksumFor(string(checksums), asset)
	if err != nil {
		return err
	}
	archive, err := download(ctx, base+"/"+asset)
	if err != nil {
		return fmt.Errorf("download %s: %w", asset, err)
	}
	actual := sha256.Sum256(archive)
	if hex.EncodeToString(actual[:]) != expected {
		return fmt.Errorf("checksum mismatch for %s", asset)
	}
	if err := os.MkdirAll(filepath.Dir(destination), 0o755); err != nil {
		return fmt.Errorf("create cache directory: %w", err)
	}
	temporary := destination + ".tmp"
	defer os.Remove(temporary)
	if strings.HasSuffix(asset, ".zip") {
		err = extractZip(archive, temporary)
	} else {
		err = extractTarGzip(archive, temporary)
	}
	if err != nil {
		return err
	}
	if err := os.Chmod(temporary, 0o755); err != nil {
		return fmt.Errorf("mark binary executable: %w", err)
	}
	if err := os.Rename(temporary, destination); err != nil {
		return fmt.Errorf("install binary: %w", err)
	}
	return nil
}

func releaseDownloadBase() string {
	if info, ok := debug.ReadBuildInfo(); ok {
		return downloadBaseForVersion(info.Main.Version)
	}
	return downloadBaseForVersion("")
}

func downloadBaseForVersion(version string) string {
	if strings.HasPrefix(version, "v") && !strings.ContainsAny(version, "/\\ \t\r\n") {
		return releasesURL + "/download/" + version
	}
	return releasesURL + "/latest/download"
}

func download(ctx context.Context, url string) ([]byte, error) {
	request, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}
	request.Header.Set("User-Agent", "rfc-agent-cli-go-bootstrap")
	response, err := http.DefaultClient.Do(request)
	if err != nil {
		return nil, err
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("unexpected HTTP status %s", response.Status)
	}
	content, err := io.ReadAll(io.LimitReader(response.Body, maxDownload+1))
	if len(content) > maxDownload {
		return nil, errors.New("download exceeds size limit")
	}
	return content, err
}

func checksumFor(checksums, asset string) (string, error) {
	for _, line := range strings.Split(checksums, "\n") {
		fields := strings.Fields(line)
		if len(fields) == 2 && strings.TrimPrefix(fields[1], "*") == asset {
			if len(fields[0]) == sha256.Size*2 {
				return strings.ToLower(fields[0]), nil
			}
			break
		}
	}
	return "", fmt.Errorf("no valid SHA-256 checksum published for %s", asset)
}

func extractTarGzip(archive []byte, destination string) error {
	gzipReader, err := gzip.NewReader(bytes.NewReader(archive))
	if err != nil {
		return fmt.Errorf("open gzip archive: %w", err)
	}
	defer gzipReader.Close()
	tarReader := tar.NewReader(gzipReader)
	for {
		header, err := tarReader.Next()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return fmt.Errorf("read tar archive: %w", err)
		}
		if header.Typeflag == tar.TypeReg && filepath.Base(header.Name) == "rfc" {
			return writeLimited(destination, tarReader)
		}
	}
	return errors.New("archive does not contain rfc")
}

func extractZip(archive []byte, destination string) error {
	reader, err := zip.NewReader(bytes.NewReader(archive), int64(len(archive)))
	if err != nil {
		return fmt.Errorf("open zip archive: %w", err)
	}
	for _, file := range reader.File {
		if filepath.Base(file.Name) != "rfc.exe" {
			continue
		}
		content, err := file.Open()
		if err != nil {
			return fmt.Errorf("open rfc.exe: %w", err)
		}
		defer content.Close()
		return writeLimited(destination, content)
	}
	return errors.New("archive does not contain rfc.exe")
}

func writeLimited(destination string, source io.Reader) error {
	file, err := os.OpenFile(destination, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0o755)
	if err != nil {
		return fmt.Errorf("create extracted binary: %w", err)
	}
	written, copyErr := io.Copy(file, io.LimitReader(source, maxDownload+1))
	closeErr := file.Close()
	if copyErr != nil {
		return fmt.Errorf("extract binary: %w", copyErr)
	}
	if closeErr != nil {
		return fmt.Errorf("close extracted binary: %w", closeErr)
	}
	if written > maxDownload {
		return errors.New("extracted binary exceeds size limit")
	}
	return nil
}
