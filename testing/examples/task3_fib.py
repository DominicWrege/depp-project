import sys


def fib2(n):
    if n == 0 or n == 1:
        return 1
    return n * fib2(n-1)

n1 = int(sys.argv[1])
n2 = int(sys.argv[2])
print("fib:", fib2(n1), fib2(n2))