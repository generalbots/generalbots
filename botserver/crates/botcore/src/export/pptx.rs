use base64::Engine;
use std::io::Write;
use zip::ZipWriter;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransitionEffect {
    Fade,
    Slide,
    Zoom,
    Wipe,
    None,
}

impl TransitionEffect {
    pub fn as_ooxml(&self) -> &'static str {
        match self {
            Self::Fade => "fade",
            Self::Slide => "slide",
            Self::Zoom => "zoom",
            Self::Wipe => "wipe",
            Self::None => "none",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "fade" => Self::Fade,
            "slide" | "slide-left" | "slide-right" => Self::Slide,
            "zoom" | "zoom-in" | "zoom-out" => Self::Zoom,
            "wipe" | "wipe-left" | "wipe-right" | "wipe-up" | "wipe-down" => Self::Wipe,
            _ => Self::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlideElement {
    pub element_type: SlideElementType,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub rotation: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum SlideElementType {
    Text {
        content: String,
        font_size: Option<u32>,
        bold: bool,
        italic: bool,
        underline: bool,
        color: Option<String>,
        font_family: Option<String>,
        alignment: Option<String>,
    },
    Image {
        data: Vec<u8>,
        mime_type: String,
        alt_text: Option<String>,
    },
    Shape {
        shape_type: String,
        fill_color: Option<String>,
        line_color: Option<String>,
        text: Option<String>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[derive(Debug, Clone)]
pub struct Slide {
    pub title: Option<String>,
    pub body: Option<String>,
    pub elements: Vec<SlideElement>,
    pub background_color: Option<String>,
    pub background_image: Option<Vec<u8>>,
    pub notes: Option<String>,
    pub transition: TransitionEffect,
    pub transition_duration: u32,
    pub layout: SlideLayout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlideLayout {
    Title,
    TitleAndContent,
    TwoContent,
    Blank,
    SectionHeader,
    Comparison,
    ContentWithCaption,
}

impl SlideLayout {
    pub fn as_ooxml_id(&self) -> u32 {
        match self {
            Self::Title => 0,
            Self::TitleAndContent => 1,
            Self::SectionHeader => 2,
            Self::TwoContent => 3,
            Self::Comparison => 4,
            Self::ContentWithCaption => 5,
            Self::Blank => 6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PptxStyles {
    pub slide_width: u32,
    pub slide_height: u32,
    pub default_font: String,
    pub heading_font: String,
    pub title_color: String,
    pub body_color: String,
    pub accent_color: String,
    pub background_color: String,
}

impl Default for PptxStyles {
    fn default() -> Self {
        Self {
            slide_width: 12192000,
            slide_height: 6858000,
            default_font: "Calibri".to_string(),
            heading_font: "Calibri Light".to_string(),
            title_color: "1F3864".to_string(),
            body_color: "333333".to_string(),
            accent_color: "2E75B6".to_string(),
            background_color: "FFFFFF".to_string(),
        }
    }
}

pub struct PptxDocument {
    pub slides: Vec<Slide>,
    pub styles: PptxStyles,
    pub author: Option<String>,
    pub title: Option<String>,
    pub subject: Option<String>,
}

impl PptxDocument {
    pub fn new() -> Self {
        Self {
            slides: Vec::new(),
            styles: PptxStyles::default(),
            author: None,
            title: None,
            subject: None,
        }
    }

    pub fn add_slide(&mut self, slide: Slide) {
        self.slides.push(slide);
    }

    pub fn add_text_slide(
        &mut self,
        title: &str,
        body: &str,
        background_color: Option<&str>,
    ) {
        let mut elements = Vec::new();
        elements.push(SlideElement {
            element_type: SlideElementType::Text {
                content: title.to_string(),
                font_size: Some(44),
                bold: true,
                italic: false,
                underline: false,
                color: Some("#1F3864".to_string()),
                font_family: Some("Calibri Light".to_string()),
                alignment: Some("center".to_string()),
            },
            x: 457200,
            y: 685800,
            width: 11277600,
            height: 914400,
            rotation: None,
        });
        elements.push(SlideElement {
            element_type: SlideElementType::Text {
                content: body.to_string(),
                font_size: Some(24),
                bold: false,
                italic: false,
                underline: false,
                color: Some("#333333".to_string()),
                font_family: Some("Calibri".to_string()),
                alignment: Some("left".to_string()),
            },
            x: 914400,
            y: 2286000,
            width: 10363200,
            height: 3657600,
            rotation: None,
        });
        self.slides.push(Slide {
            title: Some(title.to_string()),
            body: Some(body.to_string()),
            elements,
            background_color: background_color.map(|c| c.to_string()),
            background_image: None,
            notes: None,
            transition: TransitionEffect::Fade,
            transition_duration: 400,
            layout: SlideLayout::TitleAndContent,
        });
    }

    pub fn export(&self) -> Result<Vec<u8>, String> {
        let mut buf = std::io::Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(&mut buf);

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Default Extension="png" ContentType="image/png"/>
  <Default Extension="jpeg" ContentType="image/jpeg"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
  <Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
  <Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
  <Override PartName="/ppt/presProps.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presProps+xml"/>
  <Override PartName="/ppt/viewProps.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.viewProps+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
  <Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>
"#,
        )
        .map_err(|e| e.to_string())?;

        for (i, _slide) in self.slides.iter().enumerate() {
            write!(
                zip,
                r#"  <Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#,
                i + 1
            )
            .map_err(|e| e.to_string())?;
        }

        let has_notes = self.slides.iter().any(|s| s.notes.is_some());
        if has_notes {
            for (i, slide) in self.slides.iter().enumerate() {
                if slide.notes.is_some() {
                    write!(
                        zip,
                        r#"  <Override PartName="/ppt/notesSlides/notesSlide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"/>"#,
                        i + 1
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }

        writeln!(zip, "</Types>").map_err(|e| e.to_string())?;

        // _rels/.rels
        zip.start_file("_rels/.rels", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/>
</Relationships>"#,
        )
        .map_err(|e| e.to_string())?;

        // ppt/presentation.xml
        zip.start_file("ppt/presentation.xml", Default::default())
            .map_err(|e| e.to_string())?;

        let slide_id_list: String = self
            .slides
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let r_id = i + 256;
                let notes_rel = if s.notes.is_some() {
                    format!(r#"<p:notesIdLst><p:notesId id="{}" r:id="notes{}"/></p:notesIdLst>"#, i + 1, i + 1)
                } else {
                    String::new()
                };
                format!(
                    r#"    <p:sldId id="{}" r:id="rId{}"/>{}
"#,
                    i + 256,
                    r_id,
                    notes_rel
                )
            })
            .collect();

        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
                xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:sldMasterIdLst>
    <p:sldMasterId id="2147483648" r:id="rIdSlM"/>
  </p:sldMasterIdLst>
  <p:sldIdLst>
{sld_ids}  </p:sldIdLst>
  <p:sldSz cx="{}" cy="{}"/>
  <p:notesSz cx="9144000" cy="6858000"/>
  <p:defaultTextStyle>
    <a:defPPr>
      <a:defRPr sz="1800" kern="1200">
        <a:solidFill><a:schemeClr val="tx1"/></a:solidFill>
        <a:latin typeface="+mn-lt"/>
      </a:defRPr>
    </a:defPPr>
  </p:defaultTextStyle>
</p:presentation>"#,
            sld_ids = slide_id_list,
            self.styles.slide_width,
            self.styles.slide_height,
        )
        .map_err(|e| e.to_string())?;

        // ppt/_rels/presentation.xml.rels
        zip.start_file("ppt/_rels/presentation.xml.rels", Default::default())
            .map_err(|e| e.to_string())?;
        let mut pres_rels = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdSlM" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
  <Relationship Id="rIdThm" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
  <Relationship Id="rIdPrPr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/presProps" Target="presProps.xml"/>
  <Relationship Id="rIdVwPr" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/viewProps" Target="viewProps.xml"/>
"#
        );
        for (i, _slide) in self.slides.iter().enumerate() {
            let r_id = i + 256;
            pres_rels.push_str(&format!(
                r#"  <Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#,
                r_id,
                i + 1
            ));
        }
        for (i, slide) in self.slides.iter().enumerate() {
            if slide.notes.is_some() {
                pres_rels.push_str(&format!(
                    r#"  <Relationship Id="notes{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="notesSlides/notesSlide{}.xml"/>"#,
                    i + 1,
                    i + 1
                ));
            }
        }
        pres_rels.push_str("</Relationships>");
        write!(zip, "{}", pres_rels).map_err(|e| e.to_string())?;

        // ppt/slideMasters/slideMaster1.xml
        zip.start_file(
            "ppt/slideMasters/slideMaster1.xml",
            Default::default(),
        )
        .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld>
    <p:bg>
      <p:bgPr>
        <a:solidFill>
          <a:srgbClr val="{}"/>
        </a:solidFill>
      </p:bgPr>
    </p:bg>
  </p:cSld>
  <p:typeLst/>
</p:sldMaster>"#,
            self.styles.background_color
        )
        .map_err(|e| e.to_string())?;

        // ppt/slideMasters/_rels/slideMaster1.xml.rels
        zip.start_file(
            "ppt/slideMasters/_rels/slideMaster1.xml.rels",
            Default::default(),
        )
        .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLay" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
  <Relationship Id="rIdThm" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#,
        )
        .map_err(|e| e.to_string())?;

        // ppt/slideLayouts/slideLayout1.xml
        zip.start_file(
            "ppt/slideLayouts/slideLayout1.xml",
            Default::default(),
        )
        .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
             xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
             xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
             type="cust">
  <p:cSld name="Custom Layout">
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
    </p:spTree>
  </p:cSld>
</p:sldLayout>"#,
        )
        .map_err(|e| e.to_string())?;

        // ppt/slideLayouts/_rels/slideLayout1.xml.rels
        zip.start_file(
            "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
            Default::default(),
        )
        .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdThm" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#,
        )
        .map_err(|e| e.to_string())?;

        // ppt/theme/theme1.xml
        zip.start_file("ppt/theme/theme1.xml", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Default Theme">
  <a:themeElements>
    <a:clrScheme name="Default">
      <a:dk1><a:srgbClr val="000000"/></a:dk1>
      <a:lt1><a:srgbClr val="FFFFFF"/></a:lt1>
      <a:dk2><a:srgbClr val="{}"/></a:dk2>
      <a:lt2><a:srgbClr val="EEEEEE"/></a:lt2>
      <a:accent1><a:srgbClr val="{}"/></a:accent1>
      <a:accent2><a:srgbClr val="ED7D31"/></a:accent2>
      <a:accent3><a:srgbClr val="A5A5A5"/></a:accent3>
      <a:accent4><a:srgbClr val="FFC000"/></a:accent4>
      <a:accent5><a:srgbClr val="4472C4"/></a:accent5>
      <a:accent6><a:srgbClr val="70AD47"/></a:accent6>
      <a:hlink><a:srgbClr val="0563C1"/></a:hlink>
      <a:folHlink><a:srgbClr val="954F72"/></a:folHlink>
    </a:clrScheme>
    <a:fontScheme name="Default">
      <a:majorFont><a:latin typeface="{}"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
      <a:minorFont><a:latin typeface="{}"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
    </a:fontScheme>
    <a:fmtScheme name="Default">
      <a:fillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:fillStyleLst>
      <a:lnStyleLst>
        <a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
        <a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
      </a:lnStyleLst>
      <a:effectStyleLst>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
        <a:effectStyle><a:effectLst/></a:effectStyle>
      </a:effectStyleLst>
      <a:bgFillStyleLst>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
        <a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
      </a:bgFillStyleLst>
    </a:fmtScheme>
  </a:themeElements>
</a:theme>"#,
            self.styles.title_color,
            self.styles.accent_color,
            self.styles.heading_font,
            self.styles.default_font,
        )
        .map_err(|e| e.to_string())?;

        // ppt/presProps.xml
        zip.start_file("ppt/presProps.xml", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentationPr xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>
"#,
        )
        .map_err(|e| e.to_string())?;

        // ppt/viewProps.xml
        zip.start_file("ppt/viewProps.xml", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:viewPr xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:normalViewPr><p:restoredLeft sz="17067"/><p:restoredTop sz="90733"/></p:normalViewPr>
  <p:slideViewPr><p:cSldViewPr><p:cViewPrefs><p:showGuides val="0"/></p:cViewPrefs></p:cSldViewPr></p:slideViewPr>
  <p:notesTextViewPr><p:cSldViewPr/></p:notesTextViewPr>
  <p:gridSpacing cx="72000" cy="72000"/>
</p:viewPr>"#,
        )
        .map_err(|e| e.to_string())?;

        // Individual slides
        let mut image_counter = 0u32;
        for (i, slide) in self.slides.iter().enumerate() {
            let slide_num = i + 1;
            let mut rels = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rIdLay" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
"#
            );

            let mut sp_tree = String::from(r#"<p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>"#);

            for (ei, element) in slide.elements.iter().enumerate() {
                let el_id = ei + 2;
                match &element.element_type {
                    SlideElementType::Text {
                        content,
                        font_size,
                        bold,
                        italic,
                        underline,
                        color,
                        font_family,
                        alignment,
                    } => {
                        let sz = font_size.unwrap_or(18) * 100;
                        let bold_xml = if *bold { r#"<a:b/>"# } else { "" };
                        let italic_xml = if *italic { r#"<a:i/>"# } else { "" };
                        let underline_xml = if *underline { r#"<a:u/>"# } else { "" };
                        let color_val = color
                            .as_deref()
                            .unwrap_or(&self.styles.body_color)
                            .trim_start_matches('#');
                        let font = font_family
                            .as_deref()
                            .unwrap_or(&self.styles.default_font);
                        let align_val = match alignment.as_deref() {
                            Some("center") => "ctr",
                            Some("right") => "r",
                            _ => "l",
                        };
                        let escaped = content
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;")
                            .replace("\"", "&quot;")
                            .replace("'", "&apos;");
                        let cx = element.width;
                        let cy = element.height;
                        let off_x = element.x;
                        let off_y = element.y;

                        sp_tree.push_str(&format!(
                            r#"<p:sp>
        <p:nvSpPr>
          <p:cNvPr id="{}" name="TextBox{}"/>
          <p:cNvSpPr txBox="1"/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
          <a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill>
        </p:spPr>
        <p:txBody>
          <a:bodyPr rtlCol="0" anchor="t"/>
          <a:lstStyle/>
          <a:p>
            <a:pPr algn="{}"/>
            <a:r>
              <a:rPr lang="en-US" sz="{}" dirty="0" smtClean="0"{}>
                <a:solidFill><a:srgbClr val="{}"/></a:solidFill>
                <a:latin typeface="{}"/>
              </a:rPr>
              <a:t xml:space="preserve">{}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>"#,
                            el_id, el_id,
                            off_x, off_y, cx, cy,
                            align_val,
                            sz,
                            format!("{}{}{}", bold_xml, italic_xml, underline_xml),
                            color_val, font, escaped
                        ));
                    }
                    SlideElementType::Image {
                        data,
                        mime_type,
                        alt_text,
                    } => {
                        image_counter += 1;
                        let img_id = image_counter;
                        let ext = if mime_type.contains("jpeg") || mime_type.contains("jpg") {
                            "jpeg"
                        } else {
                            "png"
                        };
                        let img_path = format!("ppt/media/image{}.{}", img_id, ext);
                        rels.push_str(&format!(
                            r#"  <Relationship Id="rImg{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image{}.{}"/>"#,
                            img_id, img_id, ext
                        ));

                        zip.start_file(&img_path, Default::default())
                            .map_err(|e| e.to_string())?;
                            std::io::copy(
                            &mut std::io::Cursor::new(data),
                            &mut zip,
                        )
                        .map_err(|e| e.to_string())?;

                        let cx = element.width;
                        let cy = element.height;
                        let off_x = element.x;
                        let off_y = element.y;
                        let alt = alt_text
                            .as_deref()
                            .unwrap_or("Image")
                            .replace("&", "&amp;")
                            .replace("<", "&lt;")
                            .replace(">", "&gt;");

                        sp_tree.push_str(&format!(
                            r#"<p:pic>
        <p:nvPicPr>
          <p:cNvPr id="{}" name="Image{}" descr="{}"/>
          <p:cNvPicPr/>
          <p:nvPr/>
        </p:nvPicPr>
        <p:blipFill>
          <a:blip r:embed="rImg{}"/>
          <a:stretch><a:fillRect/></a:stretch>
        </p:blipFill>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="rect"><a:avLst/></a:prstGeom>
        </p:spPr>
      </p:pic>"#,
                            el_id, img_id, alt, img_id,
                            off_x, off_y, cx, cy
                        ));
                    }
                    SlideElementType::Shape {
                        shape_type,
                        fill_color,
                        line_color,
                        text,
                    } => {
                        let fill = fill_color
                            .as_deref()
                            .unwrap_or("2E75B6")
                            .trim_start_matches('#');
                        let cx = element.width;
                        let cy = element.height;
                        let off_x = element.x;
                        let off_y = element.y;

                        let shape_prst = match shape_type.to_lowercase().as_str() {
                            "circle" | "ellipse" | "oval" => "ellipse",
                            "triangle" => "triangle",
                            "diamond" => "diamond",
                            "arrow" | "right-arrow" => "rightArrow",
                            "line" => "line",
                            _ => "rect",
                        };

                        if let Some(txt) = text {
                            let escaped_txt = txt
                                .replace("&", "&amp;")
                                .replace("<", "&lt;")
                                .replace(">", "&gt;");
                            sp_tree.push_str(&format!(
                                r#"<p:sp>
        <p:nvSpPr>
          <p:cNvPr id="{}" name="Shape{}"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="{}"><a:avLst/></a:prstGeom>
          <a:solidFill><a:srgbClr val="{}"/></a:solidFill>
        </p:spPr>
        <p:txBody>
          <a:bodyPr rtlCol="0" anchor="ctr"/>
          <a:lstStyle/>
          <a:p>
            <a:pPr algn="ctr"/>
            <a:r>
              <a:rPr lang="en-US" sz="1800">
                <a:solidFill><a:schemeClr val="lt1"/></a:solidFill>
              </a:rPr>
              <a:t>{}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>"#,
                                el_id, el_id,
                                off_x, off_y, cx, cy,
                                shape_prst, fill, escaped_txt
                            ));
                        } else {
                            sp_tree.push_str(&format!(
                                r#"<p:sp>
        <p:nvSpPr>
          <p:cNvPr id="{}" name="Shape{}"/>
          <p:cNvSpPr/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="{}" y="{}"/>
            <a:ext cx="{}" cy="{}"/>
          </a:xfrm>
          <a:prstGeom prst="{}"><a:avLst/></a:prstGeom>
          <a:solidFill><a:srgbClr val="{}"/></a:solidFill>
        </p:spPr>
      </p:sp>"#,
                                el_id, el_id,
                                off_x, off_y, cx, cy,
                                shape_prst, fill
                            ));
                        }
                    }
                    SlideElementType::Table { headers, rows } => {
                        let cx = element.width;
                        let cy = element.height;
                        let off_x = element.x;
                        let off_y = element.y;
                        let col_count = headers.len().max(1);
                        let col_width = cx / col_count as u32;

                        let mut tbl_xml = format!(
                            r#"<p:graphicFrame>
        <p:nvGraphicFramePr>
          <p:cNvPr id="{}" name="Table{}"/>
          <p:cNvGraphicFramePr/>
          <p:nvPr/>
        </p:nvGraphicFramePr>
        <p:xfrm>
          <a:off x="{}" y="{}"/>
          <a:ext cx="{}" cy="{}"/>
        </p:xfrm>
        <a:graphic>
          <a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">
            <a:tbl>
              <a:tblPr>
                <a:tableStyleId>{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}</a:tableStyleId>
              </a:tblPr>
              <a:tblGrid>"#,
                            el_id, el_id,
                            off_x, off_y, cx, cy
                        );
                        for _ in 0..col_count {
                            tbl_xml.push_str(&format!(
                                r#"<a:gridCol w="{}"/>"#,
                                col_width
                            ));
                        }
                        tbl_xml.push_str(r#"</a:tblGrid>"#);

                        // Header row
                        tbl_xml.push_str(r#"<a:tr h="370840"><a:tc><a:txBody><a:bodyPr/><a:p><a:r><a:rPr sz="1400" b="1"><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill></a:rPr><a:t>"#);
                        for (hi, h) in headers.iter().enumerate() {
                            let escaped = h
                                .replace("&", "&amp;")
                                .replace("<", "&lt;")
                                .replace(">", "&gt;");
                            if hi > 0 {
                                tbl_xml.push_str(r#"</a:t></a:r></a:p></a:txBody></a:tc><a:tc><a:txBody><a:bodyPr/><a:p><a:r><a:rPr sz="1400" b="1"><a:solidFill><a:srgbClr val="FFFFFF"/></a:solidFill></a:rPr><a:t>"#);
                            }
                            tbl_xml.push_str(&escaped);
                        }
                        tbl_xml.push_str(r#"</a:t></a:r></a:p></a:txBody></a:tc></a:tr>"#);

                        // Data rows
                        for row in rows {
                            tbl_xml.push_str(r#"<a:tr h="370840">"#);
                            for (ci, cell) in row.iter().enumerate() {
                                let escaped = cell
                                    .replace("&", "&amp;")
                                    .replace("<", "&lt;")
                                    .replace(">", "&gt;");
                                tbl_xml.push_str(&format!(
                                    r#"<a:tc><a:txBody><a:bodyPr/><a:p><a:r><a:rPr sz="1200"><a:solidFill><a:srgbClr val="333333"/></a:solidFill></a:rPr><a:t>{esc}</a:t></a:r></a:p></a:txBody></a:tc>"#,
                                    esc = escaped
                                ));
                                if ci == 0 && row.len() < headers.len() {
                                    let extra = headers.len() - row.len();
                                    for _ in 0..extra {
                                        tbl_xml.push_str(r#"<a:tc><a:txBody><a:bodyPr/><a:p><a:r><a:rPr sz="1200"/><a:t/></a:r></a:p></a:txBody></a:tc>"#);
                                    }
                                }
                            }
                            tbl_xml.push_str(r#"</a:tr>"#);
                        }
                        tbl_xml.push_str(
                            r#"</a:tbl></a:graphicData></a:graphic></p:graphicFrame>"#,
                        );
                        sp_tree.push_str(&tbl_xml);
                    }
                }
            }

            sp_tree.push_str("</p:spTree>");

            // Slide XML
            let bg_color = slide
                .background_color
                .as_deref()
                .unwrap_or(&self.styles.background_color)
                .trim_start_matches('#');

            let transition_xml = match slide.transition {
                TransitionEffect::None => String::new(),
                _ => {
                    let dur = slide.transition_duration * 1000;
                    let trans_type = slide.transition.as_ooxml();
                    format!(
                        r#"  <p:transition spd="med" advTm="{}">
    <p:{}/>
  </p:transition>"#,
                        dur, trans_type
                    )
                }
            };

            let slide_xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
       xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld name="Slide{}">
    <p:bg>
      <p:bgPr>
        <a:solidFill>
          <a:srgbClr val="{}"/>
        </a:solidFill>
      </p:bgPr>
    </p:bg>
    {}
  </p:cSld>
  {}
</p:sld>"#,
                slide_num, bg_color, sp_tree, transition_xml
            );

            let slide_path = format!("ppt/slides/slide{}.xml", slide_num);
            zip.start_file(&slide_path, Default::default())
                .map_err(|e| e.to_string())?;
            write!(zip, "{}", slide_xml).map_err(|e| e.to_string())?;

            // Slide rels
            let rels_path = format!("ppt/slides/_rels/slide{}.xml.rels", slide_num);
            rels.push_str("</Relationships>");
            zip.start_file(&rels_path, Default::default())
                .map_err(|e| e.to_string())?;
            write!(zip, "{}", rels).map_err(|e| e.to_string())?;

            // Notes slide
            if let Some(ref notes) = slide.notes {
                let escaped_notes = notes
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;");
                let notes_path = format!("ppt/notesSlides/notesSlide{}.xml", slide_num);
                let notes_xml = format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:notes xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
         xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
         xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld name="NotesSlide{}">
    <p:spTree>
      <p:nvGrpSpPr>
        <p:cNvPr id="1" name=""/>
        <p:cNvGrpSpPr/>
        <p:nvPr/>
      </p:nvGrpSpPr>
      <p:grpSpPr/>
      <p:sp>
        <p:nvSpPr>
          <p:cNvPr id="2" name="Notes Placeholder"/>
          <p:cNvSpPr txBox="1"/>
          <p:nvPr/>
        </p:nvSpPr>
        <p:spPr>
          <a:xfrm>
            <a:off x="457200" y="457200"/>
            <a:ext cx="8229600" cy="5943600"/>
          </a:xfrm>
        </p:spPr>
        <p:txBody>
          <a:bodyPr rtlCol="0" anchor="t"/>
          <a:lstStyle/>
          <a:p>
            <a:r>
              <a:rPr lang="en-US" sz="1800">
                <a:solidFill><a:srgbClr val="333333"/></a:solidFill>
              </a:rPr>
              <a:t xml:space="preserve">{}</a:t>
            </a:r>
          </a:p>
        </p:txBody>
      </p:sp>
    </p:spTree>
  </p:cSld>
</p:notes>"#,
                    slide_num, escaped_notes
                );
                zip.start_file(&notes_path, Default::default())
                    .map_err(|e| e.to_string())?;
                write!(zip, "{}", notes_xml).map_err(|e| e.to_string())?;
            }
        }

        // docProps/core.xml
        zip.start_file("docProps/core.xml", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/"
                   xmlns:dcterms="http://purl.org/dc/terms/"
                   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:creator>{}</dc:creator>
  <dc:title>{}</dc:title>
  <dc:subject>{}</dc:subject>
  <cp:keywords>presentation slides</cp:keywords>
  <dcterms:created xsi:type="dcterms:W3CDTF">{}</dcterms:created>
</cp:coreProperties>"#,
            self.author.as_deref().unwrap_or("General Bots"),
            self.title.as_deref().unwrap_or("Presentation"),
            self.subject.as_deref().unwrap_or(""),
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ")
        )
        .map_err(|e| e.to_string())?;

        // docProps/app.xml
        zip.start_file("docProps/app.xml", Default::default())
            .map_err(|e| e.to_string())?;
        write!(
            zip,
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"
            xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes">
  <Application>General Bots</Application>
  <Slides>{}</Slides>
  <PresentationFormat>Widescreen</PresentationFormat>
  <SlideCount>{}</SlideCount>
  <NotesSlideCount>{}</NotesSlideCount>
</Properties>"#,
            self.slides.len(),
            self.slides.len(),
            self.slides.iter().filter(|s| s.notes.is_some()).count(),
        )
        .map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
        Ok(buf.into_inner())
    }
}

pub fn from_html(html: &str) -> Result<PptxDocument, String> {
    let mut doc = PptxDocument::new();
    let mut current_slide: Option<Slide> = None;

    let re_h1 = regex::Regex::new(r"<h1[^>]*>(.*?)</h1>").map_err(|e| e.to_string())?;
    let re_h2 = regex::Regex::new(r"<h2[^>]*>(.*?)</h2>").map_err(|e| e.to_string())?;
    let re_p = regex::Regex::new(r"<p[^>]*>(.*?)</p>").map_err(|e| e.to_string())?;
    let re_hr = regex::Regex::new(r"<hr[^>]*/?>").map_err(|e| e.to_string())?;
    let re_img = regex::Regex::new(r#"<img[^>]*src=["']([^"']+)["'][^>]*>"#)
        .map_err(|e| e.to_string())?;
    let re_tag = regex::Regex::new(r"<[^>]+>").map_err(|e| e.to_string())?;
    let re_br = regex::Regex::new(r"<br\s*/?>").map_err(|e| e.to_string())?;

    let mut parts: Vec<&str> = re_hr.split(html).collect();
    if parts.is_empty() {
        parts.push(html);
    }

    for part in parts {
        let mut elements: Vec<SlideElement> = Vec::new();
        let mut body_lines: Vec<String> = Vec::new();
        let mut title_text: Option<String> = None;
        let mut y_offset: u32 = 685800;

        if let Some(cap) = re_h1.captures(part) {
            let text = re_tag
                .replace_all(&cap[1], "")
                .to_string()
                .trim()
                .to_string();
            title_text = Some(text.clone());
            let text2 = text.clone();
            elements.push(SlideElement {
                element_type: SlideElementType::Text {
                    content: text2,
                    font_size: Some(44),
                    bold: true,
                    italic: false,
                    underline: false,
                    color: Some("#1F3864".to_string()),
                    font_family: Some("Calibri Light".to_string()),
                    alignment: Some("center".to_string()),
                },
                x: 457200,
                y: 685800,
                width: 11277600,
                height: 914400,
                rotation: None,
            });
            y_offset = 2286000;
        } else if let Some(cap) = re_h2.captures(part) {
            let text = re_tag
                .replace_all(&cap[1], "")
                .to_string()
                .trim()
                .to_string();
            elements.push(SlideElement {
                element_type: SlideElementType::Text {
                    content: text.clone(),
                    font_size: Some(36),
                    bold: true,
                    italic: false,
                    underline: false,
                    color: Some("2E75B6".to_string()),
                    font_family: Some("Calibri Light".to_string()),
                    alignment: Some("left".to_string()),
                },
                x: 914400,
                y: y_offset,
                width: 10363200,
                height: 685800,
                rotation: None,
            });
            y_offset += 914400;
        }

        for cap in re_img.captures_iter(part) {
            let src = cap[1].to_string();
            body_lines.push(format!("[Image: {}]", src));
        }

        for cap in re_p.captures_iter(part) {
            let raw = &cap[1];
            let with_breaks = re_br.replace_all(raw, "\n");
            let text = re_tag
                .replace_all(&with_breaks, "")
                .to_string()
                .trim()
                .to_string();
            if !text.is_empty() {
                body_lines.push(text);
            }
        }

        if !body_lines.is_empty() {
            let body_text = body_lines.join("\n\n");
            elements.push(SlideElement {
                element_type: SlideElementType::Text {
                    content: body_text.clone(),
                    font_size: Some(24),
                    bold: false,
                    italic: false,
                    underline: false,
                    color: Some("333333".to_string()),
                    font_family: Some("Calibri".to_string()),
                    alignment: Some("left".to_string()),
                },
                x: 914400,
                y: y_offset,
                width: 10363200,
                height: 3657600,
                rotation: None,
            });
        }

        doc.slides.push(Slide {
            title: title_text,
            body: if body_lines.is_empty() {
                None
            } else {
                Some(body_lines.join("\n\n"))
            },
            elements,
            background_color: None,
            background_image: None,
            notes: None,
            transition: TransitionEffect::Fade,
            transition_duration: 400,
            layout: SlideLayout::TitleAndContent,
        });
    }

    Ok(doc)
}

pub fn from_json(json: &str) -> Result<PptxDocument, String> {
    #[derive(serde::Deserialize)]
    struct JsonSlide {
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        body: Option<String>,
        #[serde(default)]
        background_color: Option<String>,
        #[serde(default)]
        notes: Option<String>,
        #[serde(default)]
        transition: Option<String>,
        #[serde(default)]
        transition_duration: Option<u32>,
        #[serde(default)]
        layout: Option<String>,
        #[serde(default)]
        elements: Option<Vec<JsonElement>>,
    }

    #[derive(serde::Deserialize)]
    #[serde(tag = "type")]
    enum JsonElement {
        #[serde(rename = "text")]
        Text {
            content: String,
            #[serde(default)]
            x: Option<u32>,
            #[serde(default)]
            y: Option<u32>,
            #[serde(default)]
            width: Option<u32>,
            #[serde(default)]
            height: Option<u32>,
            #[serde(default)]
            font_size: Option<u32>,
            #[serde(default)]
            bold: Option<bool>,
            #[serde(default)]
            italic: Option<bool>,
            #[serde(default)]
            underline: Option<bool>,
            #[serde(default)]
            color: Option<String>,
            #[serde(default)]
            font_family: Option<String>,
            #[serde(default)]
            alignment: Option<String>,
        },
        #[serde(rename = "image")]
        Image {
            data_base64: String,
            #[serde(default)]
            mime_type: Option<String>,
            #[serde(default)]
            x: Option<u32>,
            #[serde(default)]
            y: Option<u32>,
            #[serde(default)]
            width: Option<u32>,
            #[serde(default)]
            height: Option<u32>,
            #[serde(default)]
            alt_text: Option<String>,
        },
        #[serde(rename = "shape")]
        Shape {
            shape_type: String,
            #[serde(default)]
            x: Option<u32>,
            #[serde(default)]
            y: Option<u32>,
            #[serde(default)]
            width: Option<u32>,
            #[serde(default)]
            height: Option<u32>,
            #[serde(default)]
            fill_color: Option<String>,
            #[serde(default)]
            line_color: Option<String>,
            #[serde(default)]
            text: Option<String>,
        },
        #[serde(rename = "table")]
        Table {
            headers: Vec<String>,
            rows: Vec<Vec<String>>,
            #[serde(default)]
            x: Option<u32>,
            #[serde(default)]
            y: Option<u32>,
            #[serde(default)]
            width: Option<u32>,
            #[serde(default)]
            height: Option<u32>,
        },
    }

    #[derive(serde::Deserialize)]
    struct JsonRoot {
        #[serde(default)]
        slides: Vec<JsonSlide>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        subject: Option<String>,
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        slide_width: Option<u32>,
        #[serde(default)]
        slide_height: Option<u32>,
        #[serde(default)]
        default_font: Option<String>,
        #[serde(default)]
        heading_font: Option<String>,
    }

    let root: JsonRoot = serde_json::from_str(json).map_err(|e| e.to_string())?;

    let mut doc = PptxDocument::new();
    doc.title = root.title;
    doc.subject = root.subject;
    doc.author = root.author;

    if let Some(w) = root.slide_width {
        doc.styles.slide_width = w;
    }
    if let Some(h) = root.slide_height {
        doc.styles.slide_height = h;
    }
    if let Some(f) = root.default_font {
        doc.styles.default_font = f;
    }
    if let Some(f) = root.heading_font {
        doc.styles.heading_font = f;
    }

    for js in root.slides {
        let transition = js
            .transition
            .as_deref()
            .map(TransitionEffect::from_str)
            .unwrap_or(TransitionEffect::Fade);
        let layout = match js.layout.as_deref() {
            Some("title") => SlideLayout::Title,
            Some("title_and_content" | "titleAndContent") => SlideLayout::TitleAndContent,
            Some("two_content" | "twoContent") => SlideLayout::TwoContent,
            Some("section_header" | "sectionHeader") => SlideLayout::SectionHeader,
            Some("comparison") => SlideLayout::Comparison,
            Some("content_with_caption" | "contentWithCaption") => SlideLayout::ContentWithCaption,
            _ => SlideLayout::Blank,
        };

        let mut elements: Vec<SlideElement> = Vec::new();

        if let Some(ref json_elements) = js.elements {
            for je in json_elements {
                match je {
                    JsonElement::Text {
                        content,
                        x,
                        y,
                        width,
                        height,
                        font_size,
                        bold,
                        italic,
                        underline,
                        color,
                        font_family,
                        alignment,
                    } => {
                        elements.push(SlideElement {
                            element_type: SlideElementType::Text {
                                content: content.clone(),
                                font_size: *font_size,
                                bold: bold.unwrap_or(false),
                                italic: italic.unwrap_or(false),
                                underline: underline.unwrap_or(false),
                                color: color.clone(),
                                font_family: font_family.clone(),
                                alignment: alignment.clone(),
                            },
                            x: x.unwrap_or(457200),
                            y: y.unwrap_or(685800),
                            width: width.unwrap_or(11277600),
                            height: height.unwrap_or(914400),
                            rotation: None,
                        });
                    }
                    JsonElement::Image {
                        data_base64,
                        mime_type,
                        x,
                        y,
                        width,
                        height,
                        alt_text,
                    } => {
                        let data =
                            base64::Engine::decode(
                                &base64::engine::general_purpose::STANDARD,
                                data_base64,
                            )
                            .map_err(|e| format!("Base64 decode error: {}", e))?;
                        let mime = mime_type
                            .clone()
                            .unwrap_or_else(|| "image/png".to_string());
                        elements.push(SlideElement {
                            element_type: SlideElementType::Image {
                                data,
                                mime_type: mime,
                                alt_text: alt_text.clone(),
                            },
                            x: x.unwrap_or(914400),
                            y: y.unwrap_or(914400),
                            width: width.unwrap_or(3657600),
                            height: height.unwrap_or(2743200),
                            rotation: None,
                        });
                    }
                    JsonElement::Shape {
                        shape_type,
                        x,
                        y,
                        width,
                        height,
                        fill_color,
                        line_color,
                        text,
                    } => {
                        elements.push(SlideElement {
                            element_type: SlideElementType::Shape {
                                shape_type: shape_type.clone(),
                                fill_color: fill_color.clone(),
                                line_color: line_color.clone(),
                                text: text.clone(),
                            },
                            x: x.unwrap_or(914400),
                            y: y.unwrap_or(914400),
                            width: width.unwrap_or(1828800),
                            height: height.unwrap_or(1828800),
                            rotation: None,
                        });
                    }
                    JsonElement::Table {
                        headers,
                        rows,
                        x,
                        y,
                        width,
                        height,
                    } => {
                        elements.push(SlideElement {
                            element_type: SlideElementType::Table {
                                headers: headers.clone(),
                                rows: rows.clone(),
                            },
                            x: x.unwrap_or(914400),
                            y: y.unwrap_or(914400),
                            width: width.unwrap_or(8229600),
                            height: height.unwrap_or(3657600),
                            rotation: None,
                        });
                    }
                }
            }
        }

        doc.slides.push(Slide {
            title: js.title,
            body: js.body,
            elements,
            background_color: js.background_color,
            background_image: None,
            notes: js.notes,
            transition,
            transition_duration: js.transition_duration.unwrap_or(400),
            layout,
        });
    }

    Ok(doc)
}
