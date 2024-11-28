package com.example.counter

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.counter.shared_types.Event
import com.example.counter.ui.theme.CounterTheme
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

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

@Composable
fun View(core: Core = viewModel()) {
    val coroutineScope = rememberCoroutineScope()
    var checked by remember { mutableStateOf(false) }
    val cellSize = 30f

    var offsetX by remember { mutableStateOf(0f) }
    var offsetY by remember { mutableStateOf(0f) }
    Canvas(modifier = Modifier.fillMaxSize().background(color=Color.Green).pointerInput(Unit){
        detectTapGestures(onTap = { location ->
            val col = ((location.x - offsetX)/ cellSize).roundToInt()
            val row = ((location.y - offsetY)/ cellSize).roundToInt()

            val cell = listOf(row, col)
            coroutineScope.launch { core.update(Event.ToggleCell(cell)) }
        })
    }.pointerInput(Unit) {
        detectDragGestures { change, dragAmount ->
            change.consume()
            offsetX += dragAmount.x
            offsetY += dragAmount.y
        }

    } ){
        if (checked) {
            coroutineScope.launch { core.update(Event.Step()) }
        }
        val canvasQuadrantSize = size / 2F
        val h = size.height
        val w = size.width

        val nCols = (w / cellSize ).roundToInt()
        val nRows = (h / cellSize ).roundToInt()

        val cells = core.view?.life ?: listOf()
        cells.forEach{ cell ->
            val row = cell[0]
            val col = cell[1]
            drawRect(
                color = Color.Black,
                size = Size(cellSize, cellSize),
                topLeft = Offset(
                    y = cellSize * row + offsetY,
                    x = cellSize * col + offsetX,
                )
            )
        }
// draw cell borders
        repeat(nCols + 1)  { col ->
            val x: Float = cellSize*col + offsetX % cellSize
            drawLine(
                strokeWidth = 3f,
                color = Color.Black,
                start = Offset(x, y = 0f),
                end = Offset(x = x, y = h),
                colorFilter = ColorFilter.tint(Color.Black)
            )
        }
        repeat(nRows + 1) {
            val y = cellSize * it + offsetY % cellSize
            drawLine(
                strokeWidth = 3f,
                color = Color.Black,
                start = Offset(y = y, x = 0f),
                end = Offset(y = y, x = w),
                colorFilter = ColorFilter.tint(Color.Black)
            )
        }

        // Draw a rectangle
        // drawRect(color = Color.Magenta, size = canvasQuadrantSize)
        // Draw a circle
        // drawCircle(color = Color.Cyan, radius = 400f)
    }

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
        modifier = Modifier.fillMaxSize(),
    ) {


        // Text(text = "Rust Core, Kotlin Shell (Jetpack Compose)", modifier = Modifier.padding(10.dp))
        Spacer(
            modifier = Modifier.weight(1f)
        )
        Row( modifier = Modifier.fillMaxWidth().background(color=Color.White),
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
                    coroutineScope.launch { core.update(Event.Step()) }
                    checked = false
                }, colors = ButtonDefaults.buttonColors(
                    containerColor = Color.hsl(348F, 0.86F, 0.61F)
                )
            ) { Text(text = "Step", color = Color.White) }
        }
    }

}

@Preview(showBackground = true)
@Composable
fun DefaultPreview() {
    CounterTheme { View() }
}
