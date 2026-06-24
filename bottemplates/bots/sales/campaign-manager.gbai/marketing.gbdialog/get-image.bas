PARAM prompt AS STRING DESCRIPTION "Descrição da imagem desejada"
PARAM style AS STRING LIKE "modern minimalist" DESCRIPTION "Estilo visual (opcional)" OPTIONAL
PARAM width AS INTEGER LIKE 1024 DESCRIPTION "Largura da imagem" OPTIONAL
PARAM height AS INTEGER LIKE 1024 DESCRIPTION "Altura da imagem" OPTIONAL

DESCRIPTION "Gera imagens de marketing usando IA."

IF NOT prompt THEN
    TALK "Descreva a imagem que você quer gerar:"
    HEAR prompt AS STRING
END IF

TALK "🎨 Gerando imagem..."
TALK "Prompt: " + prompt

enhanced_prompt = prompt
IF style THEN
    enhanced_prompt = prompt + ", " + style + " style"
END IF

enhanced_prompt = enhanced_prompt + ", professional product photography, high quality, marketing material"

result = POST "/api/ai/image/generate", #{
    prompt: enhanced_prompt,
    width: IIF(width, width, 1024),
    height: IIF(height, height, 1024)
}

TALK "✅ **Imagem Gerada!**"
TALK "URL: " + result.image_url

TALK "O que gostaria de fazer agora?"
TALK "1. Postar no Instagram"
TALK "2. Usar em um broadcast"
TALK "3. Baixar a imagem"
TALK "4. Gerar variações"

HEAR choice AS STRING

IF choice = "1" OR choice = "instagram" THEN
    TALK "Qual legenda para o post?"
    HEAR caption AS STRING
    post_result = POST "/api/social/instagram/post", #{
        image_url: result.image_url,
        caption: caption
    }
    TALK "✅ Postado no Instagram! ID: " + post_result.post_id
ELSE IF choice = "2" OR choice = "broadcast" THEN
    TALK "Qual lista para o broadcast?"
    TALK "(Guarde a URL da imagem: " + result.image_url + ")"
ELSE IF choice = "3" OR choice = "baixar" THEN
    TALK "Baixe a imagem em: " + result.image_url
ELSE IF choice = "4" OR choice = "variações" THEN
    TALK "Gerando variações..."
    variations = POST "/api/ai/image/variations", #{
        original_url: result.image_url,
        count: 3
    }
    TALK "✅ Variações geradas:"
    FOR EACH v IN variations
        TALK "- " + v.url
    NEXT
END IF
