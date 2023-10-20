from django.utils.safestring import SafeString

from ..flags import display_lang


def test_ok():
    display = display_lang("en-US")
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
