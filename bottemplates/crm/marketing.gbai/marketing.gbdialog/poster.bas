PARAM text AS STRING DESCRIPTION "Texto principal do poster"
PARAM theme AS STRING LIKE "modern" DESCRIPTION "Tema: modern, vintage, minimal, bold"
PARAM primary_color AS STRING DESCRIPTION "Cor primária (hex, opcional)" OPTIONAL
PARAM secondary_color AS STRING DESCRIPTION "Cor secundária (hex, opcional)" OPTIONAL
PARAM logo_url AS STRING DESCRIPTION "URL do logo (opcional)" OPTIONAL

DESCRIPTION "Cria posters e materiais visuais de marketing."

IF NOT text THEN
    TALK "Qual é o texto principal do poster?"
    HEAR text AS STRING
END IF

IF NOT theme THEN
    TALK "Qual tema visual? (modern, vintage, minimal, bold)"
    HEAR theme AS STRING
END IF

TALK "🎨 Criando poster..."
TALK "Texto: " + text
TALK "Tema: " + theme

prompt = "Create a marketing poster with text: '" + text + "'. "
prompt = prompt + "Style: " + theme + " graphic design. "
prompt = prompt + "Professional, eye-catching, suitable for social media. "

IF primary_color THEN
    prompt = prompt + "Primary color: " + primary_color + ". "
END IF

IF logo_url THEN
    prompt = prompt + "Include logo at bottom. "
END IF

result = POST "/api/ai/image/generate", #{
    prompt: prompt,
    width: 1080,
    height: 1080
}

TALK "✅ **Poster Criado!**"
TALK "URL: " + result.image_url
TALK ""

TALK "Opções:"
TALK "1. Baixar"
TALK "2. Postar no Instagram"
TALK "3. Criar versão alternativa"

HEAR choice AS STRING

IF choice = "1" OR choice = "baixar" THEN
    TALK "Baixe em: " + result.image_url
ELSE IF choice = "2" OR choice = "instagram" THEN
    TALK "Qual caption?"
    HEAR caption AS STRING
    post_result = POST "/api/social/instagram/post", #{
        image_url: result.image_url,
        caption: caption
    }
    TALK "✅ Postado! " + post_result.permalink
ELSE IF choice = "3" OR choice = "alternativa" THEN
    TALK "Descreva o que gostaria de mudar:"
    HEAR feedback AS STRING
    new_prompt = prompt + " Modified based on: " + feedback
    alt_result = POST "/api/ai/image/generate", #{
        prompt: new_prompt,
        width: 1080,
        height: 1080
    }
    TALK "✅ Nova versão: " + alt_result.image_url
END IF
