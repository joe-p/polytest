import pytest


def pytest_configure(config):
    config.addinivalue_line("markers", "group_add: Addition tests")
    config.addinivalue_line("markers", "group_multiply: Multiplication tests")
