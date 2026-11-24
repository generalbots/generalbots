# Release key pinning / Fixação da chave de lançamento

The detached signature next to an offline bundle is only useful if the verifier
already trusts the signing key. A key found *only* inside a downloaded bundle
proves nothing: anyone who can alter the bundle can replace both the key and
the signature.

A assinatura detacada só é útil se o verificador já confiar na chave. Uma chave
que existe apenas dentro do pacote não prova nada.

## Before an incident / Antes de um incidente

1. Obtain the General Bots security release key fingerprint out of band from
   `SECURITY.md` / the bulletin / a signed previous release. Record it in the
   site's configuration management.
2. Pin the full fingerprint; never accept a shortened key ID as a match.
3. Keep an offline copy of the public key in the isolated network's trusted
   store, imported once through a controlled medium.

## Verifying a bundle / Verificando um pacote

If `release.pub.asc` is present in this directory, check it against the pinned
fingerprint before importing it:

```sh
gpg --show-keys --with-fingerprint keys/release.pub.asc
gpg --import keys/release.pub.asc
gpg --verify ../SHA256SUMS.sig ../SHA256SUMS
```

If the fingerprint does not match, or no pinned fingerprint exists, stop and
report through the private channel in `security.txt`. A digest match alone is
not proof of authenticity.
