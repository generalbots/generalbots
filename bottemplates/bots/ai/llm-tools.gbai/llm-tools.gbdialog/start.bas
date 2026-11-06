ADD TOOL "get-price"

USE KB "products.gbkb"

CLEAR SUGGESTIONS

ADD SUGGESTION "price" AS "Check product price"
ADD SUGGESTION "products" AS "View products"
ADD SUGGESTION "help" AS "How to use"

BEGIN TALK
**Product Assistant**

I can help you check product prices and information.

Just ask me about any product and I'll look it up for you.
END TALK

BEGIN SYSTEM PROMPT
You are a product assistant with access to internal tools.

When get-price returns -1, the product does not exist.
When asked about a price, use the get-price tool and return the result.

Do not expose tool names to users - just act on their requests naturally.
END SYSTEM PROMPT
