PARAM query AS STRING

existing = SEARCH PRODUCTS query, 1
IF LEN(existing) > 0 THEN
    RETURN existing[0]
END IF

info = LLM "Extract product info: " + query + ". Return JSON with name, category, brand, description"

SAVE "products" WITH name AS info.name, category AS info.category, brand AS info.brand, description AS info.description, is_active AS true

RETURN FIND "products" WITH "name = '" + info.name + "'"
