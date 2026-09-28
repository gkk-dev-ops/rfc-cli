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
