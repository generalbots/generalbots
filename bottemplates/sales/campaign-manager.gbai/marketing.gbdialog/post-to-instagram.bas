PARAM image_url AS STRING DESCRIPTION "URL da imagem ou caption"
PARAM caption AS STRING DESCRIPTION "Caption do post (opcional)" OPTIONAL
PARAM video_url AS STRING DESCRIPTION "URL do vídeo (para Reels, opcional)" OPTIONAL

DESCRIPTION "Posta conteúdo no Instagram."

IF NOT image_url AND NOT video_url THEN
    TALK "Forneça a URL da imagem ou vídeo para postar:"
    HEAR media_url AS STRING
ELSE
    media_url = IIF(image_url, image_url, video_url)
END IF

IF NOT caption THEN
    TALK "Qual é a caption do post?"
    HEAR caption AS STRING
END IF

TALK "📸 **Preview do Post:**"
IF image_url THEN
    TALK "Imagem: " + image_url
ELSE
    TALK "Vídeo: " + video_url
END IF
TALK "Caption: " + caption

TALK "Postar no Instagram agora?"
HEAR confirm AS BOOLEAN

IF NOT confirm THEN
    TALK "Post cancelado."
    RETURN
END IF

TALK "Postando..."

IF video_url THEN
    result = POST "/api/social/instagram/reel", #{
        video_url: video_url,
        caption: caption
    }
    TALK "🎬 **Reel postado!**"
ELSE
    result = POST "/api/social/instagram/post", #{
        image_url: media_url,
        caption: caption
    }
    TALK "📸 **Postado no Instagram!**"
END IF

TALK "Post ID: " + result.post_id
TALK "Link: " + result.permalink
TALK ""
TALK "Acompanhe o engajamento em Analytics!"
