import pytest
from django.utils.safestring import SafeString

from ..flags import display_lang


@pytest.mark.parametrize("accept_language", ["en-US", "en-US;q=1.0", "en-US,en;q=0.9"])
def test_ok(accept_language: str):
    display = display_lang(accept_language)
    assert isinstance(display, SafeString)
    assert "US.svg" in display


def test_empty():
    assert display_lang("") is None


def test_invalid_tag():
    assert display_lang("english") == "english"


def test_no_territory():
    assert display_lang("en") == "en"


def test_unknown_territory():
    assert display_lang("en-QQ") == "en-QQ"
