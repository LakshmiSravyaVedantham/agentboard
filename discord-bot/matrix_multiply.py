"""Matrix multiplication implementation."""

from typing import List

Matrix = List[List[float]]


def multiply(a: Matrix, b: Matrix) -> Matrix:
    """Multiply two matrices and return the result.

    Args:
        a: Matrix of size (m x n)
        b: Matrix of size (n x p)

    Returns:
        Result matrix of size (m x p)

    Raises:
        ValueError: If matrices have incompatible dimensions or are empty.
    """
    if not a or not a[0] or not b or not b[0]:
        raise ValueError("Matrices must not be empty")

    rows_a, cols_a = len(a), len(a[0])
    rows_b, cols_b = len(b), len(b[0])

    if cols_a != rows_b:
        raise ValueError(
            f"Incompatible dimensions: ({rows_a}x{cols_a}) and ({rows_b}x{cols_b})"
        )

    result = [[0.0] * cols_b for _ in range(rows_a)]

    for i in range(rows_a):
        for j in range(cols_b):
            for k in range(cols_a):
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
