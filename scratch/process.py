from PIL import Image
import sys

def process(image_path):
    img = Image.open(image_path).convert("RGBA")
    
    bbox = img.getbbox()
    img = img.crop(bbox)
    
    img.thumbnail((36, 36), Image.Resampling.NEAREST)
    
    width, height = img.size
    
    rust_code = "vec![\n"
    
    for y in range(0, height, 2):
        rust_code += "    Line::from(vec![\n"
        for x in range(width):
            r1, g1, b1, a1 = img.getpixel((x, y))
            if y + 1 < height:
                r2, g2, b2, a2 = img.getpixel((x, y + 1))
            else:
                r2, g2, b2, a2 = (0, 0, 0, 0)
            
            if a1 < 128 and a2 < 128:
                rust_code += "        Span::raw(\" \"),\n"
            else:
                if a1 < 128:
                    rust_code += f"        Span::styled(\"▄\", Style::default().fg(Color::Rgb({r2}, {g2}, {b2}))),\n"
                elif a2 < 128:
                    rust_code += f"        Span::styled(\"▀\", Style::default().fg(Color::Rgb({r1}, {g1}, {b1}))),\n"
                else:
                    rust_code += f"        Span::styled(\"▀\", Style::default().fg(Color::Rgb({r1}, {g1}, {b1})).bg(Color::Rgb({r2}, {g2}, {b2}))),\n"
        rust_code += "    ]),\n"
    rust_code += "]\n"
    
    with open("scratch/cat_art.rs", "w") as f:
        f.write(rust_code)
    
    print("Output written to scratch/cat_art.rs")

if __name__ == "__main__":
    process("/home/stiff/.gemini/antigravity-cli/brain/d32c5164-a3c3-4959-8c3b-2c769421105a/.user_uploaded/uploaded_media_1787027649663.png")
