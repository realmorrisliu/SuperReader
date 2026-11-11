
import Foundation
import Vision
import AppKit

let path = CommandLine.arguments[1]
guard let image = NSImage(contentsOfFile: path),
      let tiff = image.tiffRepresentation,
      let rep = NSBitmapImageRep(data: tiff),
      let cg = rep.cgImage else {
  fputs("no image\n", stderr)
  exit(2)
}
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = false
request.recognitionLanguages = ["zh-Hans", "en-US"]
let handler = VNImageRequestHandler(cgImage: cg, options: [:])
try handler.perform([request])
for line in (request.results ?? []).compactMap({ $0.topCandidates(1).first?.string }) {
  print(line)
}
