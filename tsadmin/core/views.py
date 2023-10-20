from django.http import HttpRequest, JsonResponse
from django.utils import timezone
from ipware import IpWare

from .models import LogItem


def data(request: HttpRequest):
    now = timezone.now()

    ip, _ = IpWare().get_client_ip(request.META)
    ip = ip or ""

    user_agent = request.headers.get("User-Agent", "")
    accept_language = request.headers.get("Accept-Language", "")

    LogItem.objects.create(dtm=now, ip=ip, user_agent=user_agent, accept_language=accept_language)

    return JsonResponse({"ts": now.timestamp() * 1000})
