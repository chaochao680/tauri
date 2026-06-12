import { hilog } from '@kit.PerformanceAnalysisKit';

const DOMAIN = 0x0000;

@Entry
@Component
struct TransparencyTestPage {
  @State message: string = 'Transparency Test Page';

  build() {
    Row() {
      Column() {
        Text(this.message)
          .fontSize(20)
          .fontWeight(FontWeight.Bold)
      }
      .width('100%')
      .backgroundColor('#00000000')
    }
    .height('100%')
    .backgroundColor('#00000000')
  }
}
