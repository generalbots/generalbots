DESCRIPTION "Agendar uma visita à Escola Salesiana para conhecer as instalações (Confirmar todos os dados com o usuário antes de executar a ferramenta)"
PARAM nomeResponsavel AS STRING LIKE "João Silva" DESCRIPTION "Nome do responsável"
PARAM telefone AS STRING LIKE "(21) 99999-9999" DESCRIPTION "Telefone com DDD"
PARAM email AS EMAIL LIKE "joao@example.com" DESCRIPTION "Email para contato"
PARAM dataVisita AS DATE LIKE "2026-03-15" DESCRIPTION "Data desejada para visita (formato ISO: YYYY-MM-DD)"
PARAM horario AS STRING LIKE "10:00" DESCRIPTION "Horário preferencial (formato HH:MM, entre 8h e 17h)"
PARAM numeroVisitantes AS INTEGER LIKE "3" DESCRIPTION "Número de visitantes"





id = "VIS-" + FORMAT(NOW(), "yyyyMMdd") + "-" + FORMAT(RANDOM(1000, 9999), "0000")
protocoloNumero = "VIS" + FORMAT(RANDOM(100000, 999999), "000000")
dataCadastro = FORMAT(NOW(), "yyyy-MM-dd HH:mm:ss")
status = "AGUARDANDO_CONFIRMACAO"

dataVisitaDisplay = MID(dataVisita, 9, 2) + "/" + MID(dataVisita, 6, 2) + "/" + MID(dataVisita, 1, 4)

SAVE "agendamentos_visita", id, protocoloNumero, nomeResponsavel, telefone, email, dataVisita, horario, numeroVisitantes, dataCadastro, status

SET_BOT_MEMORY("ultimo_agendamento", id)
SET_BOT_MEMORY("ultimo_protocolo_visita", protocoloNumero)

BEGIN TALK
Agendamento de Visita realizado com sucesso!

PROTOCOLO: ${protocoloNumero}
ID: ${id}

Responsável: ${nomeResponsavel}
Data: ${dataVisitaDisplay} às ${horario}
Visitantes: ${numeroVisitantes} pessoa(s)

Status: Aguardando confirmação

INFORMAÇÕES:
   • Horário de atendimento: 8h às 17h
   • A visita dura aproximadamente 1 hora
   • Estacionamento disponível no local

Contato da Secretaria:
Telefone: (21) 3333-4444
WhatsApp: (21) 99999-8888
Email: secretaria@salesianos.br

Registro salvo com sucesso e e-mail de confirmação enviado!
END TALK

BEGIN MAIL email
Subject: Confirmação de Visita - Protocolo ${protocoloNumero}

Prezado(a) ${nomeResponsavel},

Recebemos seu agendamento de visita à Escola Salesiana.

DADOS DO AGENDAMENTO:
===========================================
Protocolo: ${protocoloNumero}
ID: ${id}
Responsável: ${nomeResponsavel}
Data: ${dataVisitaDisplay} às ${horario}
Visitantes: ${numeroVisitantes} pessoa(s)
===========================================

PRÓXIMOS PASSOS:
1. Aguarde confirmação da disponibilidade
2. Compareça no horário agendado
3. Traga documento de identificação

Será um prazer recebê-lo(a)!

Atenciosamente,

Secretaria da Escola Salesiana
Tel: (21) 3333-4444 | WhatsApp: (21) 99999-8888
END MAIL

RETURN id
