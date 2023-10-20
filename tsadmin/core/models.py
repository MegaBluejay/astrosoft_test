from django.db import models


class LogItem(models.Model):
    dtm = models.DateTimeField()
    ip = models.TextField()
    user_agent = models.TextField()
    accept_language = models.TextField()
