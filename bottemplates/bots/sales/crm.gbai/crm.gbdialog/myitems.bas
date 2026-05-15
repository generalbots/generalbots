'Equais estao comigo

DESCRIPTION "Called when someone asks for items assigned to them."

products = FIND "rob.csv", "user=${username}"
text = REWRITE "Do a quick report of name, resume of history, action" ${TOYAML(products)}
TALK   "I found the following items assigned to you: ${text}"
