package main

import "testing"

func TestChecksumFor(t *testing.T) {
	const digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
	got, err := checksumFor(digest+"  rfc-linux-x86_64.tar.gz\n", "rfc-linux-x86_64.tar.gz")
	if err != nil {
		t.Fatal(err)
	}
	if got != digest {
		t.Fatalf("got %q, want %q", got, digest)
	}
}

func TestChecksumForRejectsMissingAsset(t *testing.T) {
	if _, err := checksumFor("", "rfc-linux-x86_64.tar.gz"); err == nil {
		t.Fatal("expected an error")
	}
}

func TestDownloadBaseUsesInstalledModuleVersion(t *testing.T) {
	got := downloadBaseForVersion("v0.2.0-alpha.1")
	want := releasesURL + "/download/v0.2.0-alpha.1"
	if got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestDownloadBaseFallsBackForDevelopmentBuild(t *testing.T) {
	got := downloadBaseForVersion("(devel)")
	want := releasesURL + "/latest/download"
	if got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}
