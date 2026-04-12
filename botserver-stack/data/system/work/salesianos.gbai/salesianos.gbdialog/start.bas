USE_WEBSITE("https://salesianos.br", "30d")

USE KB "carta"
USE KB "proc"


USE TOOL "inscricao"
USE TOOL "consultar_inscricao"
USE TOOL "agendamento_visita"
USE TOOL "informacoes_curso"
USE TOOL "documentos_necessarios"
USE TOOL "contato_secretaria"
USE TOOL "calendario_letivo"

ADD_SUGGESTION_TOOL "inscricao" AS "Fazer Inscrição"
ADD_SUGGESTION_TOOL "consultar_inscricao" AS "Consultar Inscrição"
ADD_SUGGESTION_TOOL "agendamento_visita" AS "Agendar Visita"
ADD_SUGGESTION_TOOL "informacoes_curso" AS "Informações de Cursos"
ADD_SUGGESTION_TOOL "documentos_necessarios" AS "Documentos Necessários"
ADD_SUGGESTION_TOOL "contato_secretaria" AS "Falar com Secretaria"
ADD_SUGGESTION_TOOL "segunda_via" AS "Segunda Via de Boleto"
ADD_SUGGESTION_TOOL "calendario_letivo" AS "Calendário Letivo"
ADD_SUGGESTION_TOOL "outros" AS "Outros"

ADD SWITCHER "tables" AS "Tabelas"
ADD SWITCHER "infographic" AS "Infográfico"
ADD SWITCHER "cards" AS "Cards"
ADD SWITCHER "list" AS "Lista"
ADD SWITCHER "comparison" AS "Comparação"
ADD SWITCHER "timeline" AS "Timeline"
ADD SWITCHER "markdown" AS "Markdown"
ADD SWITCHER "chart" AS "Gráfico"

REM Validar região para escolha de secretaria.
REM Sincronizar as bases entre o Bot e a Org.


TALK "Olá! Sou o assistente virtual da Escola Salesiana. Como posso ajudá-lo hoje com inscrições, visitas, informações sobre cursos, documentos ou calendário letivo? Você pode também escolher formatos de resposta acima da caixa de mensagem."

