from django.contrib import admin
from django.templatetags.static import static
from django.utils.html import format_html
from langcodes import Language, LanguageTagError

from .flags import flags
from .models import LogItem


@admin.register(LogItem)
class LogAdmin(admin.ModelAdmin):
    list_display = ["id", "dtm", "ip", "user_agent", "display_lang"]

    @staticmethod
    def display_lang(log: LogItem):
        langs = log.accept_language.split(",")
        if not langs:
            return None

        lang_tag = langs[0]
        try:
            language = Language.get(lang_tag)
        except LanguageTagError:
            return lang_tag
        territory = language.territory
        if not territory or territory not in flags:
            return lang_tag

        flag_url = static(f"flags/{territory}.svg")
        return format_html("<img src={} alt={}>", flag_url, territory)
