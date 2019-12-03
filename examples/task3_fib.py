import sys


def fib2(n):
    if n == 0 or n == 1:
        return 1
    return n * fib2(n-1)

n = sys.argv[1]
file_name = sys.argv[2]

f = open(file_name, "a")
a = str(fib2(int(n)))
print(a)
f.write(a)
f.close()