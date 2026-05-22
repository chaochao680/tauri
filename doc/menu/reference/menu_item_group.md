MenuItemGroup
更新时间: 2026-04-08 15:25
该组件用来展示菜单MenuItem的分组。

说明
该组件从API version 9开始支持。后续版本如有新增内容，则采用上角标单独标记该内容的起始版本。

子组件
Phone
PC/2in1
Tablet
TV
Wearable
包含MenuItem子组件。

接口
Phone
PC/2in1
Tablet
TV
Wearable
MenuItemGroup(value?: MenuItemGroupOptions)

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	MenuItemGroupOptions	否	
包含设置MenuItemGroup的标题和尾部显示信息。

未设置时，不显示标题和尾部信息。

MenuItemGroupOptions对象说明
Phone
PC/2in1
Tablet
TV
Wearable
菜单MenuItem分组的标题和尾部信息。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

名称	类型	只读	可选	说明
header	ResourceStr | CustomBuilder	否	是	
设置对应group的标题显示信息。

未设置时，不显示标题信息。

footer	ResourceStr | CustomBuilder	否	是	
设置对应group的尾部显示信息。

未设置时，不显示尾部信息。

示例
Phone
PC/2in1
Tablet
TV
Wearable
详见Menu组件示例。