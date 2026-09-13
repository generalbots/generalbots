#!/usr/bin/env python3
"""Regenerate the suite app screen SVGs from the real application source.

This is a documentation generator, so it is engineered to avoid inventing
anything. Each mockup is produced from two authoritative inputs:

1. ``botserver/src/apps/registry.rs`` -- the app list the suite actually
   renders, read directly so there is no catalogue copy to drift from it.
2. ``botui/ui/suite/<app>/`` markup -- the real region hierarchy. An app
   declares its layout as ``gb-app`` children (sidebar, header, filter band,
   main area); ``hx-get`` partials are followed and inlined, so HTMX-composed
   apps render with the controls their partials actually define. Every
   ``data-i18n`` label is resolved through ``botlib/locales/en/ui.ftl``.

Regions, labels, columns and actions are therefore real. Row payloads are
drawn as neutral skeleton blocks: the figure documents the interface without
fabricating records the product does not have.

Usage:
    python3 scripts/docs_app_screens.py --report      # extraction summary
    python3 scripts/docs_app_screens.py --check       # verify only, no writes
    python3 scripts/docs_app_screens.py               # rewrite every screen
    python3 scripts/docs_app_screens.py tasks drive   # rewrite a subset

The implementation is split by responsibility under ``scripts/docs_screens/``:
``paths`` (locations), ``markup`` (parsing and label sanitising), ``extract``
(reading an app's real regions) and ``render`` (drawing the screen).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from docs_screens.cli import main

if __name__ == "__main__":
    main()
