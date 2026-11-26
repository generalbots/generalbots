# Código-fonte dos componentes copyleft / Source of the copyleft components

General Bots Lite para Windows. Este arquivo vai no instalador, na pasta
`licenses`.

## Português

Alguns programas que o instalador traz têm licenças que exigem que o
código-fonte esteja disponível para quem recebe o programa (AGPL, MPL e
LGPL). A tabela diz, para cada um, onde está o código-fonte da versão
exata que vai no instalador.

Nenhum desses programas foi alterado pelo General Bots. A MinIO foi
compilada a partir do código original, sem mudanças, porque o autor não
publicou o executável da versão que corrige a falha de segurança
CVE-2025-62506. A receita dessa compilação também está indicada abaixo.

Se algum endereço deixar de funcionar, peça uma cópia do código-fonte
abrindo um chamado em https://github.com/generalbots/generalbots/issues.

O código do próprio General Bots é MIT, e está todo em
https://github.com/generalbots/generalbots.

## English

Some programs in this installer have licences that require their source
code to stay available to whoever receives the program (AGPL, MPL and
LGPL). The table says, for each one, where the source of the exact
version in this installer is.

General Bots changed none of them. MinIO was compiled from the original
source, unchanged, because its author published no executable for the
release that fixes CVE-2025-62506. The recipe for that build is listed
below as well.

If an address stops working, ask for a copy of the source by opening an
issue at https://github.com/generalbots/generalbots/issues.

General Bots' own code is MIT, all of it at
https://github.com/generalbots/generalbots.

## Components / Componentes

| Component | Version | Licence | Source of that exact version |
|---|---|---|---|
| MinIO server (`minio.exe`) | RELEASE.2025-10-15T17-29-55Z, commit 9e49d5e7a648f00e26f2246f4dc28e6b07f8c84a | AGPL-3.0-or-later | Source archive: https://github.com/minio/minio/archive/refs/tags/RELEASE.2025-10-15T17-29-55Z.tar.gz (SHA-256 be6d0bd3696c3a13a35f02d3a0280b64319c67918b4501c5c3d87f96d000085c). Build recipe (Go 1.24.8, `CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build -trimpath` with version flags only): https://github.com/generalbots/generalbots/blob/ad6f6b665646eadc3f86cb0e38a11cc879080feb/.github/workflows/drive-bin-v4.yml. The resulting `minio.exe` (SHA-256 e63f9c4ac8b8697ca679d4864497688c3486aa9ac9370c3569203420c25e5799) is published at https://github.com/generalbots/generalbots/releases/tag/drive-bin-v4 |
| MinIO Client (`mc.exe`) | RELEASE.2025-08-13T08-35-41Z | AGPL-3.0-or-later | https://github.com/minio/mc/tree/RELEASE.2025-08-13T08-35-41Z (upstream binary, unchanged) |
| Zitadel (`zitadel.exe`) | v4.20.1 | AGPL-3.0-only; parts Apache-2.0 and MIT (see `zitadel-LICENSING.md`) | https://github.com/zitadel/zitadel/tree/v4.20.1 (upstream binary, unchanged) |
| OpenBao (`bao.exe`) | v2.7.3 | MPL-2.0 | https://github.com/openbao/openbao/tree/v2.7.3 (upstream binary, unchanged) |
| GNU libiconv (`libiconv-2.dll`) | not yet identified | LGPL | Taken from the PostgreSQL 17.4-1 Windows binaries published by EDB. EDB's build source for this DLL is not yet located; upstream: https://www.gnu.org/software/libiconv/ |
| GNU gettext libintl (`libintl-9.dll`) | not yet identified | LGPL | As libiconv; upstream: https://www.gnu.org/software/gettext/ |

The MinIO, mc, Zitadel and OpenBao repositories are Go programs. Their
tagged source pins every dependency in `go.mod` and `go.sum`; `go mod
download` at that tag fetches them.

## Not settled yet / Ainda não resolvido

This file records where the source is. Whether this meets each licence is
a legal question that has not been answered yet:

- AGPL-3.0 §6 and MPL-2.0 §3.2: whether links to upstream tags and to the
  build recipe are enough, or whether the distributor must host a copy.
- Whether "Corresponding Source" must include the Go module dependencies
  as files.
- LGPL DLLs: their versions and the exact source of EDB's build.
