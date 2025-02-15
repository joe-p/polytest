
def pytest_configure(config):
    """Register custom markers."""
    config.addinivalue_line("markers", "group_polygon: marks tests related to polygon functionality")
    config.addinivalue_line("markers", "group_shape: marks tests related to shape functionality")
    config.addinivalue_line("markers", "group_circle: marks tests related to circle functionality")
    config.addinivalue_line("markers", "group_rectangle: marks tests related to rectangle functionality")