DESCRIPTION "Consultar status de inscrição pelo número de protocolo (Confirmar o protocolo com o usuário antes de executar)"
PARAM protocolo AS STRING LIKE "INS123456" DESCRIPTION "Número do protocolo da inscrição"


resultado = "Consulta de Inscrição - Protocolo informado: " + protocolo + ". A consulta foi registrada. Nossa equipe verificará o status da inscrição e retornará em breve. Para consultas urgentes: Telefone (21) 3333-4444, WhatsApp (21) 99999-8888, Horário Segunda a Sexta 8h às 17h, Email secretaria@salesianos.br"

TALK resultado
RETURN resultado
