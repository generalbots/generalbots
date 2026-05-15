PARAM query AS STRING

categories = FIND "categories"
similar = SEARCH PRODUCTS query, 5
result = LLM "Classify '" + query + "' into: " + categories + ". Similar: " + similar + ". Return JSON with category_id, name, confidence, brand, type"

cached = FIND "products" WITH "name LIKE '%" + query + "%'"
IF cached THEN
    RETURN cached
END IF

SAVE "products" WITH name AS query, category AS result.name, brand AS result.brand, external_metadata AS result
RETURN result
