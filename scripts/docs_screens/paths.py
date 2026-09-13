"""Repository paths this generator reads and writes."""
import os


ROOT = os.path.dirname(os.path.dirname(os.path.dirname(
    os.path.abspath(__file__))))


SUITE = os.path.join(ROOT, "botui/ui/suite")


OUT = os.path.join(ROOT, "botbook/src/assets/suite")


FTL = os.path.join(ROOT, "botlib/locales/en/ui.ftl")


REGISTRY = os.path.join(ROOT, "botserver/src/apps/registry.rs")
