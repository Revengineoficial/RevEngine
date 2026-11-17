Add-Type -AssemblyName System.Drawing
$out=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\assets\branding'))
$images=@()
foreach($size in @(512,256,128,64,48,32,16)){
  $bmp=[System.Drawing.Bitmap]::new($size,$size,[System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g=[System.Drawing.Graphics]::FromImage($bmp);$g.SmoothingMode=[System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.Clear([System.Drawing.Color]::FromArgb(16,21,34))
  $cx=$size/2.0;$cy=$size/2.0;$rad=$size*0.36;$pts=[System.Drawing.PointF[]]::new(6)
  for($i=0;$i -lt 6;$i++){$a=([Math]::PI/3*$i)-([Math]::PI/6);$pts[$i]=[System.Drawing.PointF]::new([single]($cx+$rad*[Math]::Cos($a)),[single]($cy+$rad*[Math]::Sin($a)))}
  $pen=[System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(112,225,255),[single]([Math]::Max(2,$size*0.035)));$pen.LineJoin=[System.Drawing.Drawing2D.LineJoin]::Round;$g.DrawPolygon($pen,$pts)
  $p2=[System.Drawing.Pen]::new([System.Drawing.Color]::FromArgb(150,132,255),[single]([Math]::Max(2,$size*0.05)));$p2.LineJoin=[System.Drawing.Drawing2D.LineJoin]::Round
  $g.DrawLine($p2,[single]($cx-$rad*0.34),[single]($cy-$rad*0.05),[single]$cx,[single]($cy-$rad*0.28))
  $g.DrawLine($p2,[single]$cx,[single]($cy-$rad*0.28),[single]($cx+$rad*0.34),[single]($cy-$rad*0.05))
  $g.DrawLine($p2,[single]$cx,[single]($cy-$rad*0.28),[single]$cx,[single]($cy+$rad*0.36))
  $g.DrawLine($p2,[single]($cx-$rad*0.34),[single]($cy-$rad*0.05),[single]$cx,[single]($cy+$rad*0.36))
  $ms=[IO.MemoryStream]::new();$bmp.Save($ms,[System.Drawing.Imaging.ImageFormat]::Png);$images+=,[pscustomobject]@{Size=$size;Bytes=$ms.ToArray()};$ms.Dispose()
  if($size -eq 512){$bmp.Save((Join-Path $out 'revengine-icon.png'),[System.Drawing.Imaging.ImageFormat]::Png)}
  $g.Dispose();$bmp.Dispose();$pen.Dispose();$p2.Dispose()
}
$ico=[IO.MemoryStream]::new();$w=[IO.BinaryWriter]::new($ico);$w.Write([UInt16]0);$w.Write([UInt16]1);$w.Write([UInt16]$images.Count);$offset=6+16*$images.Count
foreach($im in $images){$s=$im.Size;$wh=[byte]0;if($s -lt 256){$wh=[byte]$s};$w.Write([byte]$wh);$w.Write([byte]$wh);$w.Write([byte]0);$w.Write([byte]0);$w.Write([UInt16]1);$w.Write([UInt16]32);$w.Write([UInt32]$im.Bytes.Length);$w.Write([UInt32]$offset);$offset+=$im.Bytes.Length}
foreach($im in $images){$w.Write([byte[]]$im.Bytes)}
[IO.File]::WriteAllBytes((Join-Path $out 'revengine.ico'),$ico.ToArray());$w.Dispose();$ico.Dispose();"BRANDING_PNG_AND_MULTISIZE_ICO_CREATED"
