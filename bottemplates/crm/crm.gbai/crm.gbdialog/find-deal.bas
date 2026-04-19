PARAM query AS STRING LIKE "Acme" DESCRIPTION "Nome da empresa, pessoa ou título do deal"
PARAM return_id_only AS BOOLEAN LIKE TRUE DESCRIPTION "Se TRUE retorna apenas o ID. Se FALSE, descreve as opções ao usuário." OPTIONAL

DESCRIPTION "Procura por um deal pelo nome da empresa, título ou pessoa. Usado por outros tools para não obrigar o humano a saber UUIDs."

IF NOT return_id_only THEN return_id_only = TRUE

' Search API (assuming ?search= looks at title, contact email/name, and account name)
deals = GET "/api/crm/deals?search=" + query + "&limit=5"
count = UBOUND(deals)

IF count = 0 THEN
    TALK "Não encontrei nenhum deal relacionado a '" + query + "'."
    RETURN ""
END IF

IF count = 1 THEN
    found_deal = FIRST(deals)
    IF NOT return_id_only THEN
        TALK "Encontrei: **" + found_deal.title + "** (R$ " + FORMAT(found_deal.value, "#,##0") + ", " + found_deal.stage + ")"
    END IF
    RETURN found_deal.id
END IF

' If multiple matches found, ask user to disambiguate
TALK "Encontrei " + count + " opções para '" + query + "'. Qual delas?"
idx = 1
FOR EACH d IN deals
    TALK idx + ". **" + d.title + "** (" + d.stage + ") - R$ " + FORMAT(d.value, "#,##0")
    idx = idx + 1
NEXT d

HEAR choice

' Simple index selection logic (assuming user types "1", "2")
choice_num = TO_NUMBER(choice)
IF choice_num > 0 AND choice_num <= count THEN
    selected = deals[choice_num - 1]
    RETURN selected.id
ELSE
    ' Fallback to string matching on title
    FOR EACH d IN deals
        IF INSTR(LCASE(d.title), LCASE(choice)) > 0 THEN
            RETURN d.id
        END IF
    NEXT d
END IF

TALK "Desculpe, não consegui identificar qual você escolheu."
RETURN ""
