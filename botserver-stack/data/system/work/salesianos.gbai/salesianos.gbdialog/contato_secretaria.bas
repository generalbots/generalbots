DESCRIPTION "Enviar mensagem para a secretaria da Escola Salesiana (Confirmar todos os dados com o usuário antes de executar a ferramenta)"
PARAM nome AS STRING LIKE "João Silva" DESCRIPTION "Nome do interessado"
PARAM assunto AS STRING LIKE "Informações sobre mensalidades" DESCRIPTION "Assunto da mensagem"
PARAM mensagem AS STRING LIKE "Gostaria de saber as formas de pagamento disponíveis" DESCRIPTION "Mensagem completa"
PARAM telefone AS STRING LIKE "(21) 99999-9999" DESCRIPTION "Telefone para retorno"
PARAM email AS EMAIL LIKE "joao@example.com" DESCRIPTION "Email para contato"


id = "ATE-" + FORMAT(NOW(), "yyyyMMdd") + "-" + FORMAT(RANDOM(1000, 9999), "0000")
protocoloNumero = "ATE" + FORMAT(RANDOM(100000, 999999), "000000")
dataCadastro = FORMAT(NOW(), "yyyy-MM-dd HH:mm:ss")
status = "AGUARDANDO_RESPOSTA"

SAVE "atendimentos", id, protocoloNumero, nome, assunto, mensagem, telefone, email, dataCadastro, status

SET_BOT_MEMORY("ultimo_atendimento", id)
SET_BOT_MEMORY("ultimo_protocolo_atendimento", protocoloNumero)

BEGIN TALK
Mensagem enviada para a Secretaria!

PROTOCOLO: ${protocoloNumero}
ID: ${id}

Nome: ${nome}
Assunto: ${assunto}
Mensagem: ${mensagem}

Status: Aguardando resposta (até 2 dias úteis)

Contato da Secretaria:
Telefone: (21) 3333-4444
WhatsApp: (21) 99999-8888
Horário: Segunda a Sexta, 8h às 17h
Email: secretaria@salesianos.br

Endereço:
Rua Salesiana, 123 - Centro
CEP: 20000-000 - Rio de Janeiro/RJ

Registro salvo com sucesso e e-mail de confirmação enviado!
END TALK

BEGIN MAIL email
Subject: Confirmação de Contato - Protocolo ${protocoloNumero}

Prezado(a) ${nome},

Recebemos sua mensagem para a Secretaria da Escola Salesiana.

DADOS DO CONTATO:
===========================================
Protocolo: ${protocoloNumero}
Assunto: ${assunto}
Mensagem: ${mensagem}
===========================================

Responderemos em até 2 dias úteis.

Atenciosamente,

Secretaria da Escola Salesiana
Tel: (21) 3333-4444 | WhatsApp: (21) 99999-8888
END MAIL

RETURN id
