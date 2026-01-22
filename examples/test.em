def add(a : int, b : int) returns int
  return a + b
end

def multiply(a : int, b : int) returns int
  return a * b
end

def main
  x : int = 10
  y : int = 20
  
  sum : int = add(x, y)
  product : int = multiply(x, y)
  
  result : int = sum + product
end
