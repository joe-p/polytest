import pytest


MAX_VALUE = 2**16 - 1


def add(a: int, b: int) -> int:
    return (a + b) % (MAX_VALUE + 1)


def multiply(a: int, b: int) -> int:
    return (a * b) % (MAX_VALUE + 1)


# Polytest Suite: 16bit

# Polytest Group: multiply


@pytest.mark.group_multiply
def test_product_overflow():
    """"""
    assert multiply(MAX_VALUE, 2) == MAX_VALUE - 1


@pytest.mark.group_multiply
def test_product():
    """"""
    assert multiply(3, 4) == 12


# Polytest Group: add


@pytest.mark.group_add
def test_overflow_sum():
    """"""
    assert add(MAX_VALUE, 1) == 0


@pytest.mark.group_add
def test_sum():
    """"""
    assert add(3, 4) == 7
