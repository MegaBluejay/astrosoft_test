from django.urls import path

from .views import data

urlpatterns = [
    path("data/", data),
]
