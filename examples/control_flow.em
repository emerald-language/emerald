def test_control_flow
  x : int = 5
  
  if x > 0
    y : int = 10
  end
  
  if x < 10
    z : int = 1
  else
    z : int = 2
  end
  
  counter : int = 0
  while counter < 5
    counter = counter + 1
  end
end
