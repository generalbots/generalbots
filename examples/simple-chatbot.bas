' Example Regular Bot: Simple Chatbot
USE KB "faq"

TALK "Hello! How can I help you today?"

WHEN user_says "help" DO
  TALK "I can help you with orders, returns, and general questions."
END WHEN

WHEN user_says "order status" DO
  order_id = ASK "What's your order number?"
  status = CALL TOOL "check-order" WITH order_id
  TALK "Your order status is: " + status
END WHEN

WHEN user_says "goodbye" DO
  TALK "Thank you for contacting us! Have a great day!"
END WHEN
