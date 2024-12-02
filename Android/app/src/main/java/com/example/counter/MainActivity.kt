package com.example.counter

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsBottomHeight
import androidx.compose.foundation.layout.windowInsetsTopHeight
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.counter.shared_types.Event
import com.example.counter.ui.theme.CounterTheme
import kotlinx.coroutines.launch
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.roundToInt
import kotlin.math.sin

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            CounterTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    View()
                }
            }
        }
    }
}


fun Offset.rotateBy(angle: Float): Offset {
    val angleInRadians = angle * (PI / 180)
    val cos = cos(angleInRadians)
    val sin = sin(angleInRadians)
    return Offset((x * cos - y * sin).toFloat(), (x * sin + y * cos).toFloat())
}

@Composable
fun View(core: Core = viewModel()) {
    val coroutineScope = rememberCoroutineScope()
    var checked by remember { mutableStateOf(false) }

    var cameraOffset by remember { mutableStateOf(Offset.Zero) }
    // var offsetY by remember { mutableStateOf(0f) }
    var zoom by remember { mutableStateOf(1f) }
    val cellSize = 30f
    val cellOffset = Offset(cellSize, cellSize)
    var csize by remember { mutableStateOf(Size(100f, 100f)) }
    val h = csize.height
    val w = csize.width
    val quadrent = Offset(w/2f, h/2f)


    Canvas(modifier = Modifier.fillMaxSize().background(color=Color.Green).pointerInput(Unit){
        detectTapGestures(onTap = { location ->
            // val worldPos = Offset(col*cellSize, row*cellSize)
            // val location = (worldPos + cameraOffset )*zoom + Offset(w/2f, h/2f)
            val worldPos = (location - quadrent) / zoom - cameraOffset - cellOffset/2f
            val index = (worldPos / cellSize)
            val col = index.x.roundToInt()
            val row = index.y.roundToInt()

            val cell = listOf(row, col)
            coroutineScope.launch { core.update(Event.ToggleCell(cell)) }
        })
    }.pointerInput(Unit) {
        detectTransformGestures(onGesture = { centroid, pan, gestureZoom, gestureRotate ->
            val oldScale = zoom
            val newScale = zoom * gestureZoom
            val factor = (newScale / oldScale)
            val center = cameraOffset + Offset(csize.width, csize.height)*zoom /2f

            cameraOffset += pan /oldScale
            //offset = offset + centroid / oldScale-  centroid / newScale + pan / oldScale
            // offset = offset + pan
            // offset += centroid / oldScale - centroid / newScale
            zoom = newScale
        })
    } ){
        if (checked) {
            coroutineScope.launch { core.update(Event.Step()) }
        }
        val canvasQuadrantSize = size / 2F
        csize = size



        val cells = core.view?.life ?: listOf()
        cells.forEach{ cell ->
            val row = cell[0]
            val col = cell[1]
            val worldPos = Offset(col*cellSize, row*cellSize)

            val screenPos = (worldPos + cameraOffset )*zoom + Offset(w/2f, h/2f)

            drawRect(
                color = Color.Black,
                size = Size(cellSize, cellSize)*zoom,
                topLeft = Offset(
                    y = screenPos.y,
                    x = screenPos.x,
                )
            )
        }
        // draw cell borders
        if (zoom > 0.4){
            val screenCell = cellSize*zoom
            val nCols = (w / screenCell ).roundToInt()
            val nRows = (h / screenCell ).roundToInt()
            repeat(nCols + 1)  { col ->
                val x: Float = screenCell*col + (cameraOffset.x * zoom)  % screenCell + (w/2f) % screenCell
                drawLine(
                    strokeWidth = 3f,
                    color = Color.Black,
                    start = Offset(x, y = 0f),
                    end = Offset(x = x, y = h),
                    colorFilter = ColorFilter.tint(Color.Black)
                )
            }
            repeat(nRows + 1) { row ->
                val y: Float = screenCell*row + (cameraOffset.y * zoom)  % screenCell + (h/2f) % screenCell
                drawLine(
                    strokeWidth = 3f,
                    color = Color.Black,
                    start = Offset(y = y, x = 0f),
                    end = Offset(y = y, x = w),
                    colorFilter = ColorFilter.tint(Color.Black)
                )
            }
        }


        // Draw a rectangle
        // drawRect(color = Color.Magenta, size = canvasQuadrantSize)
        // Draw a circle
        // drawCircle(color = Color.Cyan, radius = 400f)
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier.imePadding()
    ) {
        Spacer(modifier = Modifier.fillMaxWidth().windowInsetsTopHeight(
            WindowInsets.systemBars).background(Color.Black))

        Text( text = core.alert ?: "")

        // Text(text = "Rust Core, Kotlin Shell (Jetpack Compose)", modifier = Modifier.padding(10.dp))
        Spacer(
            modifier = Modifier.weight(1f)
        )
        val whiteTint = Color.hsl(1f,1f,1f,0.8f)

        Row( modifier = Modifier.fillMaxWidth().background(color=whiteTint),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically
            ) {

            Row(
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically

            ){
                Checkbox( checked=checked, onCheckedChange = {checked = it})
                Text(text = "Play", color = Color.Black)
            }

            Button(
                modifier = Modifier.padding(15.dp),
                onClick = {
                    coroutineScope.launch {
                        core.update(Event.Step())
                        core.update(Event.Echo("derp"))
                    }
                    checked = false
                }, colors = ButtonDefaults.buttonColors(
                    containerColor = Color.hsl(348F, 0.86F, 0.61F)
                )
            ) { Text(text = "Step", color = Color.White) }
        }
        Spacer(modifier = Modifier.fillMaxWidth().windowInsetsBottomHeight(
            WindowInsets.systemBars))
    }

}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    CounterTheme { View() }
}
