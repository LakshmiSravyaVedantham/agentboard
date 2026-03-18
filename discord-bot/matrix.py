"""Matrix multiplication implementation."""


def multiply(a: list[list[float]], b: list[list[float]]) -> list[list[float]]:
    """Multiply two matrices a and b.

    Args:
        a: Matrix of dimensions m x n.
        b: Matrix of dimensions n x p.

    Returns:
        Resulting matrix of dimensions m x p.

    Raises:
        ValueError: If matrices have incompatible dimensions.
    """
    if not a or not b:
        raise ValueError("Matrices must not be empty")

    n = len(a[0])
    if any(len(row) != n for row in a):
        raise ValueError("Matrix a has inconsistent row lengths")
    if len(b) != n:
        raise ValueError(
            f"Incompatible dimensions: a has {n} columns, b has {len(b)} rows"
        )
    p = len(b[0])
    if any(len(row) != p for row in b):
        raise ValueError("Matrix b has inconsistent row lengths")

    m = len(a)
    result = [[0.0] * p for _ in range(m)]
    for i in range(m):
        for j in range(p):
            for k in range(n):
                result[i][j] += a[i][k] * b[k][j]
    return result


if __name__ == "__main__":
    a = [
        [1, 2, 3],
        [4, 5, 6],
    ]
    b = [
        [7, 8],
        [9, 10],
        [11, 12],
    ]
    result = multiply(a, b)
    for row in result:
        print(row)
