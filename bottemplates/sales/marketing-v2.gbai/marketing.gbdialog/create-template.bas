PARAM name AS STRING LIKE "Promoção de Páscoa" DESCRIPTION "Nome do template"
PARAM channel AS STRING LIKE "whatsapp" DESCRIPTION "Canal: whatsapp, email, sms, telegram"
PARAM content AS STRING DESCRIPTION "Conteúdo do template (corpo da mensagem ou HTML para email)"
PARAM header_image AS STRING DESCRIPTION "URL da imagem de cabeçalho (opcional)" OPTIONAL
PARAM ai_prompt AS STRING LIKE "Escreva uma mensagem amigável oferecendo 15% de desconto em nossos produtos" DESCRIPTION "Prompt para IA gerar o conteúdo (opcional)" OPTIONAL
PARAM meta_template_id AS STRING DESCRIPTION "ID do template approval pela META (para WhatsApp)" OPTIONAL

DESCRIPTION "Cria um Template de Marketing (email, WhatsApp, SMS) que pode ser usado em campanhas."

IF ai_prompt THEN
    TALK "🤖 Gerando conteúdo com IA..."
    generated_content = LLM ai_prompt
    content = generated_content
    TALK "✅ Conteúdo gerado:"
    TALK generated_content
    TALK "Deseja usar este conteúdo ou pedir para a IA gerar outro?"
    HEAR use_content AS BOOLEAN
    IF NOT use_content THEN
        TALK "Por favor, forneça o conteúdo do template:"
        HEAR content AS STRING
    END IF
END IF

IF channel = "whatsapp" AND NOT meta_template_id THEN
    TALK "⚠️ Para WhatsApp, você precisa de um template approval pela META."
    TALK "Deseja proseguir salvando como rascunho (sem meta_template_id)?"
    HEAR proceed AS BOOLEAN
    IF NOT proceed THEN
        RETURN
    END IF
END IF

new_template = POST "/api/marketing/templates", #{
    name: name,
    channel: channel,
    content: content,
    header_image: header_image,
    ai_prompt: ai_prompt,
    meta_template_id: meta_template_id,
    status: IIF(meta_template_id, "approved", "draft")
}

TALK "📝 **Template Criado!**"
TALK "Nome: " + name
TALK "Canal: " + UCASE(channel)
TALK "Status: " + IIF(meta_template_id, "Approved", "Draft")
TALK "ID: " + new_template.id

IF channel = "whatsapp" AND meta_template_id THEN
    TALK "✅ Template pronto para uso em broadcasts WhatsApp!"
END IF

RETURN new_template.id
