# Código-fonte dos componentes copyleft / Source of the copyleft components

General Bots Lite para Windows. Este arquivo vai no instalador, na pasta
`licenses`.

## Português

Alguns programas que o instalador traz, ou que o aplicativo baixa quando um
serviço é ligado, têm licenças que exigem que o código-fonte esteja
disponível para quem recebe o programa (AGPL, MPL e LGPL). As tabelas dizem,
para cada um, onde está o código-fonte da versão exata.

Nenhum desses programas foi alterado pelo General Bots. O armazenamento de
objetos é o OtterIO (RELEASE.2026-11-25T20-30-00Z) e a ferramenta de linha de
comando é a OC CLI (RELEASE.2026-11-25T20-30-01Z), ambos Apache-2.0, com os
endereços e digests exatos indicados abaixo. Nenhum dos dois está sob AGPL.

Se algum endereço deixar de funcionar, peça uma cópia do código-fonte
abrindo um chamado em https://github.com/generalbots/generalbots/issues.

O código do próprio General Bots é MIT, e está todo em
https://github.com/generalbots/generalbots.

## English

Some programs that this installer carries, or that the app downloads when a
service is switched on, have licences that require their source code to stay
available to whoever receives the program (AGPL, MPL and LGPL). The tables
say, for each one, where the source of the exact version is.

General Bots changed none of them. The object store is OtterIO
(RELEASE.2026-11-25T20-30-00Z) and the command-line tool is OC CLI
(RELEASE.2026-11-25T20-30-01Z), both Apache-2.0; their exact addresses and
digests are listed below. Neither is under AGPL.

If an address stops working, ask for a copy of the source by opening an
issue at https://github.com/generalbots/generalbots/issues.

General Bots' own code is MIT, all of it at
https://github.com/generalbots/generalbots.

## Inside the installer / Dentro do instalador

| Component | Version | Licence | Source of that exact version |
|---|---|---|---|
| OtterIO Object Storage (`otterio.exe`, component `drive`) | RELEASE.2026-11-25T20-30-00Z | Apache-2.0 | Windows: https://github.com/soulteary/otterio/releases/download/RELEASE.2026-11-25T20-30-00Z/otterio-windows-amd64.exe (SHA-256 25049f76f49ac6e988eb8481c6a94f8632a8494e5554b68dc98c1aa2163e6a01). Linux x64: https://github.com/soulteary/otterio/releases/download/RELEASE.2026-11-25T20-30-00Z/otterio-linux-amd64 (cd8cfcc59f31829665adba12abe70ec79308245a3f7b2bb46376917f617b3aa7). ARM64: https://github.com/soulteary/otterio/releases/download/RELEASE.2026-11-25T20-30-00Z/otterio-linux-arm64 (5a034416033b39345cbc93ef41d8be5e4185c3ac3a008bf498a66de2c7ab3366). Tagged source: soulteary/otterio, tag RELEASE.2026-11-25T20-30-00Z. Licence text: `LICENSE` in the payload |
| OC CLI (`oc.exe`, component `mc`) | RELEASE.2026-11-25T20-30-01Z | Apache-2.0 | Windows: https://github.com/soulteary/oc/releases/download/RELEASE.2026-11-25T20-30-01Z/oc-RELEASE.2026-11-25T20-30-01Z-windows-amd64.zip (SHA-256 3630178855befd300f4baf3933652af03f156d577cfc5f654a3d619516f77881). Linux x64: https://github.com/soulteary/oc/releases/download/RELEASE.2026-11-25T20-30-01Z/oc-RELEASE.2026-11-25T20-30-01Z-linux-amd64.tar.gz (b410af77b1473da1c14cf5cecf3684087162da150d50f0c59034c25b4a25794c). ARM64: https://github.com/soulteary/oc/releases/download/RELEASE.2026-11-25T20-30-01Z/oc-RELEASE.2026-11-25T20-30-01Z-linux-arm64.tar.gz (3847a0352c397539f31806064400d1d00dac65d3c3cc96f56268cf83533e0186). Tagged source: soulteary/oc, tag RELEASE.2026-11-25T20-30-01Z. Licence text: `LICENSE` in the payload |
| OpenBao (`bao.exe`) | v2.7.3 | MPL-2.0 | https://github.com/openbao/openbao/tree/v2.7.3 (upstream binary, unchanged) |
| GNU libiconv (`libiconv-2.dll` in `installers`) | 1.15, read from the file's version stamp | LGPL-2.0-or-later (text: `LGPL-2.0.txt`) | The DLL comes from the PostgreSQL 17.4-1 Windows binaries published by EDB. Upstream source of version 1.15: https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.15.tar.gz (5,264,188 bytes), signature https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.15.tar.gz.sig . SHA-256: not hashed (GNU publishes the signature only; the tarball was not downloaded). The source EDB built from is not located |
| GNU libiconv (`libiconv-2.dll` inside the PostgreSQL 18.6-1 zip, `pgsql/bin`) | 1.19, read from the file's version stamp | LGPL-2.1-or-later (text: `LGPL-2.1.txt`) | The zip is the PostgreSQL 18.6-1 Windows binaries published by EDB, carried whole. Upstream source of version 1.19: https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.19.tar.gz (5,921,103 bytes), signature https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.19.tar.gz.sig . SHA-256: not hashed. The source EDB built from is not located |
| GNU gettext libintl (`libintl-9.dll` in `installers`, and the same file inside the PostgreSQL 18.6-1 zip) | 0.19.8, read from the file's version stamp | LGPL-2.1-or-later (text: `LGPL-2.1.txt`) | From the same EDB binaries. Upstream source of version 0.19.8: https://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.8.tar.gz (19,977,131 bytes), signature https://ftp.gnu.org/pub/gnu/gettext/gettext-0.19.8.tar.gz.sig . GNU's release announcement (https://lists.gnu.org/archive/html/info-gnu/2016-06/msg00003.html) gives MD5 7ad5c90e32ac6828de955a0432ab4a7c and SHA-1 4b3dc947a729fac8a14bde473454730919382b23 for that file. SHA-256: not hashed (GNU published none; the tarball was not downloaded). The source EDB built from is not located |

## Not in the installer: downloaded only if the service is switched on / Fora do instalador: baixado só se o serviço for ligado

| Component | Version | Licence | Source of that exact version |
|---|---|---|---|
| Zitadel (`zitadel.exe`) | v4.20.1 | AGPL-3.0-only; parts Apache-2.0 and MIT (see `zitadel-LICENSING.md`) | https://github.com/zitadel/zitadel/tree/v4.20.1 (upstream binary, unchanged, fetched from https://github.com/zitadel/zitadel/releases/download/v4.20.1/zitadel-windows-amd64.tar.gz) |

Zitadel stays in this file although the installer no longer carries it
(since 2026-11-27). The directory service is switched off in this edition,
but the app still downloads exactly this version when the service is
switched on (`GB_SERVICES_ON=directory`), and installers before that date
carried it, so a copy can be on a computer that was updated.

O Zitadel continua neste arquivo, embora o instalador não o traga mais
(desde 2026-11-27). O serviço de diretório vem desligado nesta edição, mas o
aplicativo ainda baixa exatamente esta versão quando o serviço é ligado
(`GB_SERVICES_ON=directory`), e os instaladores anteriores a essa data o
traziam; por isso pode haver uma cópia num computador que foi atualizado.

Zitadel and OpenBao are Go programs: their tagged source pins every
dependency in `go.mod` and `go.sum`; `go mod download` at that tag fetches
them. OtterIO and OC publish their tagged source at the release addresses
above.

How the versions and licences of the two GNU libraries were established
(version stamps, the sentences of the GNU release notes and licence files,
with their addresses) is in `LICENSES-desktop.md`, section 1.2.

## Not settled yet / Ainda não resolvido

This file records where the source is. Whether this meets each licence is
a legal question that has not been answered yet:

- AGPL-3.0 §6 and MPL-2.0 §3.2: whether links to upstream tags are enough,
  or whether the distributor must host a copy.
- Whether "Corresponding Source" must include the Go module dependencies
  as files.
- LGPL DLLs: the versions are known now. The exact source and build recipe
  EDB used are not: EDB's own licence list inside the 18.6-1 zip
  (`pgsql/commandlinetools_3rd_party_licenses.txt`) gives the LGPL 2.1 text
  for "gettext, libiconv, pthreads" and no source address or written offer.
- libiconv 1.15: two files of its Windows DLL build carry a GPL header in
  that release, which its maintainer later called an oversight and changed
  (https://lists.gnu.org/archive/html/bug-gnu-libiconv/2019-05/msg00003.html).
  Whether EDB's build includes them is not known.
- The PostgreSQL 18.6-1 zip is carried whole, with pgAdmin 4 and
  StackBuilder inside. EDB's list for pgAdmin
  (`pgsql/pgAdmin_3rd_party_licenses.txt`) names further LGPL parts ("QT,
  gettext, libiconv, pthreads"). Their versions and source were not checked.
- Whether a download that the app starts from an upstream release page
  (Zitadel) counts as distribution by General Bots.

## NOTICE A — source offer / aviso de código-fonte

pt-BR: "Se algum endereço deixar de funcionar, ou se você quiser uma cópia do código-fonte de qualquer componente (incluindo OpenBao e as bibliotecas GNU sob LGPL), abra um chamado em https://github.com/generalbots/generalbots/issues. Enviaremos o código da versão exata indicada neste registro. Este aviso é uma declaração de distribuição, não um parecer jurídico."

en: "If an address stops working, or if you want a copy of the source of any component (including OpenBao and the GNU LGPL libraries), open an issue at https://github.com/generalbots/generalbots/issues. We will provide the source of the exact version listed in this record. This notice is a disclosure, not a legal opinion."

## Outros avisos / Other notices

pt-BR: O aviso para dados de governo (NOTICE B) e o aviso sobre o download de CUDA (NOTICE C) estão na seção 10 de `LICENSES-desktop.md`. Os tarballs de código-fonte GNU sob LGPL e o inventário dos zips da EDB estão nas seções 1.2 e 1.1 desse mesmo arquivo.

en: The government-data notice (NOTICE B) and the CUDA-download notice (NOTICE C) are in section 10 of `LICENSES-desktop.md`. The GNU LGPL source tarballs and the EDB zip inventory are in sections 1.2 and 1.1 of that file.
