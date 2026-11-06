PARAM image AS STRING

description = SEE image
similar = SEARCH PRODUCTS description, 5

IF LEN(similar) > 0 THEN
    RETURN similar[0]
END IF

product = LLM "Extract product info from: " + description + ". Return JSON with name, brand, category, color, material"
SAVE "products" WITH name AS product.name, brand AS product.brand, category AS product.category, color AS product.color, material AS product.material

RETURN FIND "products" WITH "name = '" + product.name + "'"
