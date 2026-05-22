StatusBarViewExtensionAbility（状态栏扩展Ability）
更新时间: 2026-04-20 14:34
StatusBarViewExtensionAbility为状态栏扩展Ability，继承自UIExtensionAbility，用于给应用提供接入状态栏图标左键业务弹窗的能力。

说明
本模块接口仅可在Stage模型下使用。

系统能力： SystemCapability.PCService.StatusBarManager

起始版本： 5.0.0(12)

导入模块
PC/2in1
import { StatusBarViewExtensionAbility } from '@kit.DeskTopExtensionKit';
示例：

import { StatusBarViewExtensionAbility } from '@kit.DeskTopExtensionKit';
import { UIExtensionContentSession, Want } from '@kit.AbilityKit';

let TAG = "MyStatusBarViewAbility";

export default class MyStatusBarViewAbility extends StatusBarViewExtensionAbility {
  onCreate() {
    console.info(TAG, `onCreate`);
  }

  onSessionCreate(want: Want, session: UIExtensionContentSession) {
    console.info(TAG, `onSessionCreate, want: ${want.abilityName}`);
    session.loadContent('pages/Index');
  }

  onForeground() {
    console.info(TAG, `onForeground`);
  }

  onBackground() {
    console.info(TAG, `onBackground`);
  }

  onSessionDestroy(session: UIExtensionContentSession) {
    console.info(TAG, `onSessionDestroy`);
  }

  onDestroy() {
    console.info(TAG, `onDestroy`);
  }
}