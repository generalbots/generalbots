PARAM product_id AS STRING

product = PRODUCT product_id
IF NOT product THEN
    RETURN WITH error AS "Product not found"
END IF

IF product.description AND product.brand THEN
    RETURN product
END IF

query = product.name + " " + product.category
links = SCRAPE_ALL "https://www.google.com/search?q=" + query, "a"
links = FIRST links, 10

enriched = []
FOR i = 0 TO 2
    IF links[i] THEN
        content = SCRAPE links[i], "body"
        PUSH enriched, content
    END IF
NEXT

result = LLM "Analyze these product descriptions: " + enriched + ". Create best description for: " + product.name + ". Return JSON with description, brand, material, features"

UPDATE "products" WITH description AS result.description, brand AS result.brand, material AS result.material, external_metadata AS result WHERE id = product_id

RETURN PRODUCT product_id
