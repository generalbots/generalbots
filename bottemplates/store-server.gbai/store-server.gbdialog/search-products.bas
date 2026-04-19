PARAM query AS STRING

cached = FIND "products" WITH "name ILIKE '%" + query + "%' OR description ILIKE '%" + query + "%'"
IF cached THEN
    RETURN cached
END IF

result = SEARCH PRODUCTS query, 10
IF LEN(result) = 0 THEN
    web = SCRAPE_ALL "https://www.google.com/search?q=" + query + "+product", ".g"
    result = LLM "Extract products from: " + web + ". Return JSON array with name, price, description"
END IF

enhanced = LLM "Add descriptions: " + result + ". Return JSON array with id, name, price, description, stock"

FOR EACH item IN enhanced
    SAVE "products" WITH name AS item.name, description AS item.description, price AS item.price, external_metadata AS item
NEXT

RETURN enhanced
