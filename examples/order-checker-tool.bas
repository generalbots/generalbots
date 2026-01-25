' Example Tool: Simple Order Checker
USE TOOL "database"

WHEN called WITH order_id DO
  order = GET order FROM database WHERE id = order_id
  
  IF order.exists THEN
    RETURN order.status, order.amount, order.date
  ELSE
    RETURN "not_found", 0, ""
  END IF
END WHEN
