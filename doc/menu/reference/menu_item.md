MenuItem
更新时间: 2026-04-08 15:25
用来展示菜单中具体的菜单选项。

说明
该组件从API version 9开始支持。后续版本如有新增内容，则采用上角标单独标记该内容的起始版本。

子组件
Phone
PC/2in1
Tablet
TV
Wearable
无

接口
Phone
PC/2in1
Tablet
TV
Wearable
MenuItem(value?: MenuItemOptions | CustomBuilder)

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	MenuItemOptions | CustomBuilder	否	包含设置MenuItem的各项信息。 如果不传该参数，则创建空的MenuItem对象。
MenuItemOptions对象说明
Phone
PC/2in1
Tablet
TV
Wearable
Menu中具体item菜单项信息。

系统能力： SystemCapability.ArkUI.ArkUI.Full

名称	类型	只读	可选	说明
startIcon	ResourceStr	否	是	
MenuItem的起始图标。不支持Symbol图标。使用Symbol图标时，须使用symbolStartIcon。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

content	ResourceStr	否	是	
MenuItem的内容。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

endIcon	ResourceStr	否	是	
MenuItem的末尾图标。不支持Symbol图标。使用Symbol图标时，须使用symbolEndIcon。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

labelInfo	ResourceStr	否	是	
MenuItem结束的标签信息，如快捷方式Ctrl+C等。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

builder	CustomBuilder	否	是	
用于构建二级菜单。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

symbolStartIcon	SymbolGlyphModifier	否	是	
MenuItem起始的Symbol图标。配置该项时，原先startIcon图标不显示。

元服务API： 从API version 12开始，该接口支持在元服务中使用。

symbolEndIcon	SymbolGlyphModifier	否	是	
MenuItem末尾的Symbol图标。配置该项时，原先endIcon图标不显示。

元服务API： 从API version 12开始，该接口支持在元服务中使用。

属性
Phone
PC/2in1
Tablet
TV
Wearable
除支持通用属性外，还支持以下属性：

selected
Phone
PC/2in1
Tablet
TV
Wearable
selected(value: boolean)

设置菜单项是否选中。

从API version 10开始，该参数支持$$双向绑定变量。

从API version 18开始，该参数支持!!双向绑定变量。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	boolean	是	
菜单项是否选中。

true：菜单项被选中；false：菜单项不被选中。

默认值：false

selectIcon
Phone
PC/2in1
Tablet
TV
Wearable
selectIcon(value: boolean | ResourceStr | SymbolGlyphModifier)

设置当菜单项被选中时，是否显示被选中的图标。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	boolean | ResourceStr| SymbolGlyphModifier	是	
菜单项被选中时，是否显示被选中的图标。

true：显示默认的对勾图标；false：不显示图标。

ResourceStr：显示指定的图标。

SymbolGlyphModifier：显示指定的HMSymbol图标。

默认值：false

contentFont
Phone
PC/2in1
Tablet
TV
Wearable
contentFont(value: Font)

设置菜单项中内容信息的字体样式。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	Font	是	菜单项中内容信息的字体样式。
contentFontColor
Phone
PC/2in1
Tablet
TV
Wearable
contentFontColor(value: ResourceColor)

设置菜单项中内容信息的字体颜色。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	ResourceColor	是	
菜单项中内容信息的字体颜色。

默认值：'#E5000000'

labelFont
Phone
PC/2in1
Tablet
TV
Wearable
labelFont(value: Font)

设置菜单项中标签信息的字体样式。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	Font	是	菜单项中标签信息的字体样式。
labelFontColor
Phone
PC/2in1
Tablet
TV
Wearable
labelFontColor(value: ResourceColor)

设置菜单项中标签信息的字体颜色。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
value	ResourceColor	是	
菜单项中标签信息的字体颜色。

默认值：'#99000000'

事件
Phone
PC/2in1
Tablet
TV
Wearable
onChange
Phone
PC/2in1
Tablet
TV
Wearable
onChange(callback: (selected: boolean) => void)

当选中状态发生变化时，触发该回调。只有手动触发且MenuItem状态改变时才会触发onChange回调。

元服务API： 从API version 11开始，该接口支持在元服务中使用。

系统能力： SystemCapability.ArkUI.ArkUI.Full

参数：

参数名	类型	必填	说明
selected	boolean	是	
选中状态发生变化时，触发该回调。

true：未选中切换为选中；false：选中切换为未选中。

示例
Phone
PC/2in1
Tablet
TV
Wearable
详见Menu组件示例。