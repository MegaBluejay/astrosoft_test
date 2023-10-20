from django.contrib import admin
from django.utils.safestring import SafeString

from .flags import display_lang
from .models import LogItem


@admin.register(LogItem)
class LogAdmin(admin.ModelAdmin):
    list_display = ["id", "dtm", "ip", "user_agent", "display_lang"]

    @staticmethod
    def display_lang(log: LogItem) -> None | str | SafeString:
        return display_lang(log.accept_language)
